use std::sync::Arc;

use axum::{
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::StatusCode,
    response::IntoResponse,
};
use blastview::{
    session::{LiveSession, patch::Patch},
    view::View,
};
use futures::{SinkExt, StreamExt};
use uuid::Uuid;

use crate::state::AppState;

pub async fn live_handler<V, F>(
    ws: WebSocketUpgrade,
    Path(session_id): Path<String>,
    State(state): State<Arc<AppState<V, F>>>,
) -> impl IntoResponse
where
    V: View + Send + Sync + 'static,
    F: Fn() -> V + Send + Sync + 'static,
{
    let Some(session) = Uuid::parse_str(&session_id)
        .ok()
        .and_then(|id| state.sessions.remove(&id))
    else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    ws.on_upgrade(async |socket| handle_ws::<V, F>(socket, session.1).await)
        .into_response()
}

async fn handle_ws<V, F>(socket: WebSocket, session: (LiveSession, flume::Receiver<Patch>))
where
    V: View + Send + Sync + 'static,
    F: Fn() -> V + Send + Sync,
{
    let (session, patch_rx) = session;
    let session = Arc::new(session);
    Arc::clone(&session).begin_re_render_task();

    let (mut sender, mut receiver) = socket.split();

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
