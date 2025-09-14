mod handlers;

use std::{io, sync::Arc};

use axum::{Router, routing::get};
use blastview::view::View;
use tokio::net::TcpListener;

pub async fn serve<V, F>(factory: F) -> io::Result<()>
where
    V: View + Send + Sync + 'static,
    F: Fn() -> V + Send + Sync + 'static,
{
    let factory = Arc::new(factory);

    let app = Router::new()
        .route("/", get(handlers::catch_all::<V, F>))
        .route("/{*path}", get(handlers::catch_all::<V, F>))
        .with_state(factory);

    let listener = TcpListener::bind(("0.0.0.0", 8080)).await?;
    axum::serve(listener, app).await
}
