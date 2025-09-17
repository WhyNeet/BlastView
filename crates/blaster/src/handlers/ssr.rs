use std::sync::Arc;

use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use blastview::{renderer::StaticRenderer, view::View};

pub async fn ssr_handler<V, F>(State(factory): State<Arc<F>>) -> impl IntoResponse
where
    V: View + Send + Sync + 'static,
    F: Fn() -> V + Send + Sync,
{
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
        StaticRenderer::render_to_string(|| factory()),
        include_str!("../js/script.js")
    );

    tracing::debug!("serving ssr content");

    Html(html)
}
