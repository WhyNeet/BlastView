use std::sync::Arc;

use axum::{Router, routing::get};
use blastview::view::View;

pub mod live;
pub mod ssr;

pub fn router<V, F>() -> Router<Arc<F>>
where
    V: View + Send + Sync + 'static,
    F: Fn() -> V + Send + Sync + 'static,
{
    Router::new()
        .route("/__ws", get(live::live_handler))
        .route("/", get(ssr::ssr_handler))
        .route("/{*path}", get(ssr::ssr_handler))
}
