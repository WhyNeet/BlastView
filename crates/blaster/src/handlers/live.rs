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
use tokio_util::sync::CancellationToken;
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

    let token = CancellationToken::new();

    let recv_task = tokio::spawn(async move {
        while let Some(Ok(message)) = receiver.next().await {
            match message {
                Message::Text(event) => {
                    let event = event.as_str().to_string();
                    session.dispatch_event(event);
                }
                Message::Close(_) => {
                    break;
                }
                _ => {}
            }
        }
    });

    let send_task_token = token.clone();
    tokio::spawn(async move {
        loop {
            tokio::select! {
              Ok(patch) = patch_rx.recv_async() => {
                sender
                    .send(Message::Text(serde_json::to_string(&patch).unwrap().into()))
                    .await
                    .unwrap();
              },
              _ = send_task_token.cancelled() => {
                break;
              }
            }
        }
    });

    recv_task.await.unwrap();
    token.cancel();
}
