use axum::extract::ws::{Message, WebSocket};
use axum::extract::WebSocketUpgrade;
use axum::http::StatusCode;
use axum::response::*;
use axum::routing::get;
use axum::{Router, Server};

use chrono::{NaiveDate, Utc};

use earendel::EarendelState;

use heimdall::{HeimdallServer, HeimdallState};

use tokio::sync::broadcast;
use tokio::sync::Mutex;
use tokio::task;

use tracing::trace;

use std::error::Error;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();

    let earendel_state = Arc::new(Mutex::new(None));

    let (heimdall_tx, _) = broadcast::channel::<HeimdallState>(1);

    let heimdall_server = HeimdallServer::new();
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
        .route("/earendel.html", get(earendel_get))
        .route("/earendel.mjs", get(earendel_script_get))
        .route(
            "/earendel/apod",
            get({
                let shared_state = Arc::clone(&earendel_state);
                move || earendel_apod(shared_state)
            }),
        )
        .route("/heimdall.html", get(heimdall_get))
        .route("/heimdall.mjs", get(heimdall_script_get))
        .route(
            "/ws/heimdall",
            get(move |upgrade| heimdall_ws(upgrade, heimdall_tx)),
        );

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

#[cfg(feature = "production")]
async fn get_file(path: &str) -> String {
    include_str!(path)
}

async fn root_get() -> Html<String> {
    let markup = get_file("index.html").await;

    Html(markup)
}

async fn earendel_get() -> Html<String> {
    let markup = get_file("earendel.html").await;

    Html(markup)
}

async fn earendel_script_get() -> Response<String> {
    let script = get_file("earendel.mjs").await;

    Response::builder()
        .header("content-type", "application/javascript;charset=utf-8")
        .body(script)
        .unwrap()
}

async fn earendel_apod(
    state: Arc<Mutex<Option<(NaiveDate, EarendelState)>>>,
) -> Result<String, StatusCode> {
    let mut cached_state = state.lock().await;

    // cache is invalidated after UTC date changes
    let today = Utc::now().date_naive();
    let new_state = match cached_state.as_ref() {
        Some((date, apod)) if date == &today => apod.to_owned(),
        Some(_) | None => earendel::get_apod_image()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    };
    let payload =
        serde_json::to_string(&new_state).expect("earendel_apod could not serialize state");
    *cached_state = Some((today, new_state));

    Ok(payload)
}

async fn heimdall_get() -> Html<String> {
    let markup = get_file("heimdall.html").await;

    Html(markup)
}

async fn heimdall_script_get() -> Response<String> {
    let script = get_file("heimdall.mjs").await;

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
