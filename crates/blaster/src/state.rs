use std::sync::Arc;

use blastview::view::RenderableView;
use dashmap::DashMap;
use uuid::Uuid;
use web::session::{LiveSession, patch::Patch};

pub struct AppState<V: RenderableView + Send + Sync + 'static, F: Fn() -> V + Send + Sync + 'static>
{
    pub factory: Arc<F>,
    pub sessions: DashMap<Uuid, (LiveSession, flume::Receiver<Patch>)>,
}

impl<V: RenderableView + Send + Sync + 'static, F: Fn() -> V + Send + Sync + 'static>
    AppState<V, F>
{
    pub fn new(factory: F) -> Self {
        Self {
            factory: Arc::new(factory),
            sessions: Default::default(),
        }
    }
}
