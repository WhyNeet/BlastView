use std::sync::Arc;

use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
};
use blastview::{
    session::{LiveSession, patch::Patch},
    view::View,
};
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
    let (session, patch_rx) = LiveSession::new(|| factory());
    let session = Arc::new(session);
    Arc::clone(&session).begin_re_render_task();

    let (mut sender, mut receiver) = socket.split();

    sender
        .send(Message::Text(
            serde_json::to_string(&Patch::ReplaceInner {
                selector: "#app".to_string(),
                html: session.dynamic_render(),
            })
            .unwrap()
            .into(),
        ))
        .await
        .unwrap();

    let recv_task = tokio::spawn(async move {
        while let Some(Ok(message)) = receiver.next().await {
            let event = message.to_text().unwrap();
            session.dispatch_event(event.to_string());
        }
    });

    let send_task = tokio::spawn(async move {
        while let Ok(patch) = patch_rx.recv() {
            sender
                .send(Message::Text(serde_json::to_string(&patch).unwrap().into()))
                .await
                .unwrap();
        }
    });

    recv_task.await.unwrap();
    send_task.await.unwrap();
}
