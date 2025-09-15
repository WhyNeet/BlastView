use std::sync::Arc;

use axum::{
    extract::{State, WebSocketUpgrade, ws::WebSocket},
    response::IntoResponse,
};
use blastview::{session::LiveSession, view::View};

pub async fn live_handler<V, F>(
    ws: WebSocketUpgrade,
    State(factory): State<Arc<F>>,
) -> impl IntoResponse
where
    V: View + Send + Sync + 'static,
    F: Fn() -> V + Send + Sync + 'static,
{
    ws.on_upgrade(async |socket| handle_ws(socket, factory).await)
}

async fn handle_ws<V, F>(mut socket: WebSocket, factory: Arc<F>)
where
    V: View + Send + Sync + 'static,
    F: Fn() -> V + Send + Sync,
{
    let session = LiveSession::new(|| factory());

    tokio::task::spawn(async move {
        while let Some(Ok(message)) = socket.recv().await {
            let event = message.to_text().unwrap();
            session.dispatch_event(event.to_string());
        }
    })
    .await
    .unwrap();
}
