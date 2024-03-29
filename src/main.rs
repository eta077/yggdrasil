use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Path, Query, WebSocketUpgrade};
use axum::http::{Response, StatusCode};
use axum::response::*;
use axum::routing::get;
use axum::{Router, Server};

use earendel::EarendelServer;

use heimdall::{HeimdallServer, HeimdallState};

use pulldown_cmark::html::push_html;
use pulldown_cmark::Parser;

use serde::{Deserialize, Serialize};

use tokio::sync::broadcast;
use tokio::sync::Mutex;
use tokio::task;

use tower_http::trace::TraceLayer;

use tracing::*;

use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Serialize)]
struct BlogContent {
    markup: String,
    previous: Option<String>,
    next: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BlogParams {
    selection: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let earendel_server = Arc::new(Mutex::new(EarendelServer::new()));

    let (heimdall_tx, _) = broadcast::channel::<HeimdallState>(1);

    let mut heimdall_server = HeimdallServer::new();
    let heimdall_receiver = heimdall_server.add_listener();
    let heimdall_sender = heimdall_tx.clone();
    task::spawn_blocking(move || loop {
        if let Ok(new_state) = heimdall_receiver.recv() {
            trace!("yggdrasil setting state to {:?}", new_state);
            let _ = heimdall_sender.send(new_state);
        }
    });
    heimdall_server.start();

    let router = Router::new()
        .route("/", get(root_get))
        .route("/blog", get(blog_get))
        .route("/blog.mjs", get(blog_script_get))
        .route("/blog/content", get(blog_content))
        .route("/earendel", get(earendel_get))
        .route("/earendel.mjs", get(earendel_script_get))
        .route(
            "/earendel/apod",
            get({
                let shared_state = Arc::clone(&earendel_server);
                move || earendel_apod(shared_state)
            }),
        )
        .route(
            "/earendel/apod-fits",
            get({
                let shared_state = Arc::clone(&earendel_server);
                move || earendel_apod_fits(shared_state)
            }),
        )
        .route("/heimdall", get(heimdall_get))
        .route("/heimdall.mjs", get(heimdall_script_get))
        .route(
            "/ws/heimdall",
            get(move |upgrade| heimdall_ws(upgrade, heimdall_tx)),
        )
        .route("/heimdall/assets/:image", get(heimdall_image_get))
        .layer(TraceLayer::new_for_http());

    let server = Server::bind(&"0.0.0.0:7032".parse()?).serve(router.into_make_service());
    server.await?;

    Ok(())
}

#[cfg(feature = "debug")]
async fn get_file(path: &str) -> String {
    let mut real_path = String::from("src/");
    real_path += path;
    tokio::fs::read_to_string(real_path).await.unwrap()
}

async fn root_get() -> Html<String> {
    #[cfg(feature = "debug")]
    let markup = get_file("index.html").await;

    #[cfg(feature = "production")]
    let markup = include_str!("index.html").to_owned();

    Html(markup)
}

async fn blog_get() -> Html<String> {
    #[cfg(feature = "debug")]
    let markup = get_file("blog/blog.html").await;

    #[cfg(feature = "production")]
    let markup = include_str!("blog/blog.html").to_owned();

    Html(markup)
}

async fn blog_script_get() -> Response<String> {
    #[cfg(feature = "debug")]
    let script = get_file("blog/blog.mjs").await;

    #[cfg(feature = "production")]
    let script = include_str!("blog/blog.mjs").to_owned();

    Response::builder()
        .header("content-type", "application/javascript;charset=utf-8")
        .body(script)
        .unwrap()
}

async fn blog_content(params: Option<Query<BlogParams>>) -> Result<String, StatusCode> {
    let blog_dir = tokio::task::spawn_blocking(get_blog_dir)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let blog_count = blog_dir.len();
    if blog_count == 0 {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    let (file_path, previous, next) = match params {
        Some(params) => match blog_dir.iter().position(|path| {
            path.to_str()
                .unwrap_or_default()
                .contains(&params.0.selection)
        }) {
            Some(index) => {
                let previous = if index > 0 {
                    Some(
                        blog_dir[index - 1]
                            .file_name()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default()
                            .to_owned(),
                    )
                } else {
                    None
                };
                let next = if index < blog_count - 1 {
                    Some(
                        blog_dir[index + 1]
                            .file_name()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default()
                            .to_owned(),
                    )
                } else {
                    None
                };
                (blog_dir[index].to_owned(), previous, next)
            }
            None => return Err(StatusCode::NOT_FOUND),
        },
        None => {
            let previous = if blog_count > 1 {
                Some(
                    blog_dir[blog_count - 2]
                        .file_name()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default()
                        .to_owned(),
                )
            } else {
                None
            };
            (blog_dir[blog_count - 1].to_owned(), previous, None)
        }
    };

    let markdown = tokio::fs::read_to_string(file_path).await.unwrap();
    let parser = Parser::new(&markdown);
    let mut markup = String::new();
    push_html(&mut markup, parser);
    let clean_markup = ammonia::clean(&markup);

    let blog_content = BlogContent {
        markup: clean_markup,
        previous,
        next,
    };

    let payload =
        serde_json::to_string(&blog_content).expect("blog_content could not serialize data");

    Ok(payload)
}

fn get_blog_dir() -> Result<Vec<PathBuf>, String> {
    let mut files = std::fs::read_dir("blog/content")
        .map_err(|_| String::from("could not read blog directory"))?
        .filter(|res| res.is_ok())
        .map(|e| e.unwrap().path())
        .collect::<Vec<PathBuf>>();

    files.sort();

    Ok(files)
}

async fn earendel_get() -> Html<String> {
    #[cfg(feature = "debug")]
    let markup = get_file("earendel/earendel.html").await;

    #[cfg(feature = "production")]
    let markup = include_str!("earendel/earendel.html").to_owned();

    Html(markup)
}

async fn earendel_script_get() -> Response<String> {
    #[cfg(feature = "debug")]
    let script = get_file("earendel/earendel.mjs").await;

    #[cfg(feature = "production")]
    let script = include_str!("earendel/earendel.mjs").to_owned();

    Response::builder()
        .header("content-type", "application/javascript;charset=utf-8")
        .body(script)
        .unwrap()
}

async fn earendel_apod(server: Arc<Mutex<EarendelServer>>) -> Result<String, StatusCode> {
    let state = server.lock().await.get_apod_image().await.map_err(|e| {
        error!("{e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let payload = serde_json::to_string(&state).expect("earendel_apod could not serialize APOD state");

    Ok(payload)
}

async fn earendel_apod_fits(server: Arc<Mutex<EarendelServer>>) -> Result<String, StatusCode> {
    let state = server
        .lock()
        .await
        .get_fits_for_apod(1)
        .await
        .map_err(|e| {
            error!("{e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("FITS result: {state:?}");

    let payload =
        serde_json::to_string(&state).expect("earendel_apod_fits could not serialize FITS state");

    Ok(String::from(payload))
}

async fn heimdall_get() -> Html<String> {
    #[cfg(feature = "debug")]
    let markup = get_file("heimdall/heimdall.html").await;

    #[cfg(feature = "production")]
    let markup = include_str!("heimdall/heimdall.html").to_owned();

    Html(markup)
}

async fn heimdall_script_get() -> Response<String> {
    #[cfg(feature = "debug")]
    let script = get_file("heimdall/heimdall.mjs").await;

    #[cfg(feature = "production")]
    let script = include_str!("heimdall/heimdall.mjs").to_owned();

    Response::builder()
        .header("content-type", "application/javascript;charset=utf-8")
        .body(script)
        .unwrap()
}

async fn heimdall_ws(
    upgrade: WebSocketUpgrade,
    sender: broadcast::Sender<HeimdallState>,
) -> impl IntoResponse {
    upgrade.on_upgrade(|ws| async { heimdall_stream(ws, sender).await })
}

async fn heimdall_stream(mut ws: WebSocket, sender: broadcast::Sender<HeimdallState>) {
    let mut receiver = sender.subscribe();

    while let Ok(state) = receiver.recv().await {
        let payload =
            serde_json::to_string(&state).expect("heimdall_stream could not serialize state");
        if let Err(e) = ws.send(Message::Text(payload)).await {
            // this error is expected if the client is closed/reloaded
            trace!("heimdall_stream failed to send message: {e}");
            break;
        }
    }
}

async fn heimdall_image_get(Path(image): Path<String>) -> Result<Vec<u8>, StatusCode> {
    let mut path = if cfg!(feature = "debug") {
        String::from("src/")
    } else {
        String::new()
    };
    path = path + "assets/heimdall/" + &image;
    tokio::fs::read(path).await.map_err(|e| {
        error!("{e}");
        StatusCode::NOT_FOUND
    })
}
