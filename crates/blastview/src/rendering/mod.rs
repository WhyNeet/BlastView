use std::{collections::HashSet, sync::Mutex};

use uuid::Uuid;

#[derive(Default)]
pub struct RenderingQueue {
    pub render_queue: Mutex<HashSet<Uuid>>,
    pub deferred_queue: Mutex<HashSet<Uuid>>,
}

impl RenderingQueue {
    pub(crate) fn enqueue(&self, id: Uuid) {
        if let Ok(mut queue) = self.render_queue.try_lock() {
            queue.insert(id);
        } else {
            self.deferred_queue.lock().unwrap().insert(id);
        }
    }

    pub fn clear(&self) {
        self.render_queue.lock().unwrap().clear();
        self.deferred_queue.lock().unwrap().clear();
    }
}
