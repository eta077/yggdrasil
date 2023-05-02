use axum::extract::ws::{Message, WebSocket};
use axum::extract::WebSocketUpgrade;
use axum::http::{Response, StatusCode};
use axum::response::{Form, Html, IntoResponse, Redirect};
use axum::routing::get;
use axum::{Router, Server};

use axum_extra::extract::cookie::{Cookie, Key, PrivateCookieJar};

use christpoint_kids::{KidsServer, LoginParams};

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

    let kids_server = Arc::new(Mutex::new(KidsServer {}));
    let kids_key = Key::generate();

    let router = Router::new()
        .route("/", get(root_get))
        .route("/earendel", get(earendel_get))
        .route("/earendel.mjs", get(earendel_script_get))
        .route(
            "/earendel/apod",
            get({
                let shared_state = Arc::clone(&earendel_state);
                move || earendel_apod(shared_state)
            }),
        )
        .route("/heimdall", get(heimdall_get))
        .route("/heimdall.mjs", get(heimdall_script_get))
        .route(
            "/ws/heimdall",
            get(move |upgrade| heimdall_ws(upgrade, heimdall_tx)),
        )
        .route("/kids", get(kids_get))
        .route(
            "/kids/login",
            get(kids_login_get).post({
                let shared_state = Arc::clone(&kids_server);
                move |jar, params| kids_login(shared_state, jar, params)
            }),
        )
        .route("/kids/login.mjs", get(kids_login_script_get))
        .with_state(kids_key);

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
    let markup = get_file("earendel/earendel.html").await;

    Html(markup)
}

async fn earendel_script_get() -> Response<String> {
    let script = get_file("earendel/earendel.mjs").await;

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
    let markup = get_file("heimdall/heimdall.html").await;

    Html(markup)
}

async fn heimdall_script_get() -> Response<String> {
    let script = get_file("heimdall/heimdall.mjs").await;

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

async fn kids_get(jar: PrivateCookieJar) -> (StatusCode, Html<String>) {
    if let Some(id) = jar.get("userId") {
        let markup = get_file("kids/kids.html").await;
        (StatusCode::OK, Html(markup))
    } else {
        let markup = get_file("kids/unauth.html").await;

        (StatusCode::UNAUTHORIZED, Html(markup))
    }
}

async fn kids_login_get() -> Html<String> {
    let markup = get_file("kids/login.html").await;

    Html(markup)
}

async fn kids_login_script_get() -> Response<String> {
    let script = get_file("kids/login.mjs").await;

    Response::builder()
        .header("content-type", "application/javascript;charset=utf-8")
        .body(script)
        .unwrap()
}

async fn kids_login(
    kids_server: Arc<Mutex<KidsServer>>,
    jar: PrivateCookieJar,
    Form(params): Form<LoginParams>,
) -> Result<(PrivateCookieJar, Redirect), (StatusCode, String)> {
    let user = kids_server.lock().await.login(params).map_err(|e| {
        let text = match e {
            christpoint_kids::LoginError::InternalError => {
                String::from("An error occurred during the login process")
            }
            christpoint_kids::LoginError::InvalidCredentials => {
                String::from("Invalid credentials, please try again.")
            }
        };
        (StatusCode::BAD_REQUEST, text)
    })?;
    let new_jar = jar.add(Cookie::new("userId", user.id.to_owned()));
    Ok((new_jar, Redirect::to("/kids")))
}

// TODO:
// key loading
// encrypt email/password before sending?
//
