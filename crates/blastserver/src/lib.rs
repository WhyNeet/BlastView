mod handlers;

use std::{io, sync::Arc};

use axum::{Router, routing::get};
use blastview::component::Component;
use tokio::net::TcpListener;

pub async fn serve<C>(component: C) -> io::Result<()>
where
    C: Component + Send + Sync + 'static,
{
    let component = Arc::new(component);

    let app = Router::new()
        .route("/", get(handlers::catch_all::<C>))
        .route("/{*path}", get(handlers::catch_all::<C>))
        .with_state(component);

    let listener = TcpListener::bind(("0.0.0.0", 8080)).await?;
    axum::serve(listener, app).await
}
