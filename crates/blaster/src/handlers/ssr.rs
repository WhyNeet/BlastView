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
                const root = document.getElementById("app");
                const ws = new WebSocket("http://127.0.0.1:8080/__ws");
                ws.addEventListener("message", (e) => {{
                const html = e.data;
                root.innerHTML = html;
                setupEventListeners();
                }});
                function setupEventListeners() {{
                            document.querySelectorAll('[data-id]').forEach(element => {{
                                const events = element.dataset.events.split(",");
                                events.forEach(eventType => element.addEventListener(eventType, handleEvent));
                            }});
                        }}

                        function handleEvent(e) {{
                            const dataId = e.target.getAttribute('data-id');
                            const eventType = e.type;
                            const eventId = `${{dataId}}_${{eventType}}`;

                            if (ws.readyState === WebSocket.OPEN) {{
                                ws.send(eventId);
                            }}
                        }}
                </script>
            </body>
            </html>
        "#,
        StaticRenderer::render_to_string(|| factory())
    );

    tracing::debug!("serving ssr content");

    Html(html)
}
