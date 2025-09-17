mod handlers;
mod state;

use std::{io, sync::Arc};

use axum::Router;
use blastview::view::View;
use tokio::net::TcpListener;

use crate::state::AppState;

pub async fn serve<V, F>(factory: F) -> io::Result<()>
where
    V: View + Send + Sync + 'static,
    F: Fn() -> V + Send + Sync + 'static,
{
    let state = AppState::new(factory);

    let app = Router::new()
        .merge(handlers::router::<V, F>())
        .with_state(Arc::new(state));

    let listener = TcpListener::bind(("0.0.0.0", 8080)).await?;
    tracing::info!("Listening on 0.0.0.0:8080");
    axum::serve(listener, app).await
}
