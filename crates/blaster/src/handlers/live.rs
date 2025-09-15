use std::sync::Arc;

use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
};
use blastview::{session::LiveSession, view::View};
use futures::{SinkExt, StreamExt};

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

async fn handle_ws<V, F>(socket: WebSocket, factory: Arc<F>)
where
    V: View + Send + Sync + 'static,
    F: Fn() -> V + Send + Sync,
{
    let session = LiveSession::new(|| factory());

    let (mut sender, mut receiver) = socket.split();

    sender
        .send(Message::Text(session.dynamic_render().into()))
        .await
        .unwrap();

    let recv_task = tokio::spawn(async move {
        while let Some(Ok(message)) = receiver.next().await {
            let event = message.to_text().unwrap();
            session.dispatch_event(event.to_string());
        }
    });

    let send_task = tokio::spawn(async move {});

    recv_task.await.unwrap();
    send_task.await.unwrap();
}
