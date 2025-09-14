use std::sync::Arc;

use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use blastview::{renderer::Renderer, view::View};

pub async fn catch_all<V, F>(State(factory): State<Arc<F>>) -> impl IntoResponse
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
          </body>
          </html>
      "#,
        Renderer::render_to_string(|| factory())
    );

    Html(html)
}
