use std::sync::Arc;

use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use blastview::{component::Component, renderer};

pub async fn catch_all<C>(State(component): State<Arc<C>>) -> impl IntoResponse
where
    C: Component + Send + Sync + 'static,
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
        renderer::render_component_to_string(component.as_ref())
    );

    Html(html)
}
