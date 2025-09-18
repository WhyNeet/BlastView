use std::sync::Mutex;

use uuid::Uuid;

#[derive(Default)]
pub struct RenderingQueue {
    pub render_queue: Mutex<Vec<Uuid>>,
    pub deferred_queue: Mutex<Vec<Uuid>>,
}

impl RenderingQueue {
    pub(crate) fn enqueue(&self, id: Uuid) {
        if let Ok(mut queue) = self.render_queue.try_lock() {
            queue.push(id);
        } else {
            self.deferred_queue.lock().unwrap().push(id);
        }
    }

    pub fn clear(&self) {
        self.render_queue.lock().unwrap().clear();
        self.deferred_queue.lock().unwrap().clear();
    }
}
