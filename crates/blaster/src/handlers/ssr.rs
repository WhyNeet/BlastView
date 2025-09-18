use std::sync::Arc;

use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use blastview::view::View;
use uuid::Uuid;
use web::session::LiveSession;

use crate::state::AppState;

pub async fn ssr_handler<V, F>(State(state): State<Arc<AppState<V, F>>>) -> impl IntoResponse
where
    V: View + Send + Sync + 'static,
    F: Fn() -> V + Send + Sync,
{
    let factory = Arc::clone(&state.factory);
    let session_id = Uuid::new_v4();
    let session = LiveSession::new(|| factory());
    let html = session.0.dynamic_render();
    state.sessions.insert(session_id, session);
    let hydration_script =
        include_str!("../js/script.js").replace("$SESSION_ID", &session_id.to_string());

    let html = format!(
        r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>BlastView App</title>
            </head>
            <body>
                <div id="app">{}</div>
                <script>
                {}
                </script>
            </body>
            </html>
        "#,
        html, hydration_script
    );

    tracing::debug!("serving ssr content");

    Html(html)
}
