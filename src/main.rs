use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::Html;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use axum::{Router, Server};

use heimdall::{HeimdallServer, HeimdallState};

use tokio::sync::broadcast;
use tokio::task;

use tracing::trace;

use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();

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
        .route("/heimdall.html", get(heimdall_get))
        .route("/heimdall.mjs", get(heimdall_script_get))
        .route("/ws/heimdall", get(heimdall_ws))
        .with_state(heimdall_tx);

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
    State(sender): State<broadcast::Sender<HeimdallState>>,
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
