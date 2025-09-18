use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

pub struct Effect {
    hash: AtomicU64,
    cleanup: Mutex<Option<Box<dyn FnOnce() + Send + Sync>>>,
}

impl Effect {
    pub fn new<T: Hash>(
        f: impl (FnOnce() -> Box<dyn FnOnce() + Send + Sync>) + Send + Sync,
        deps: T,
    ) -> Self {
        Self {
            cleanup: Mutex::new(Some(f())),
            hash: {
                let mut hasher = DefaultHasher::new();
                deps.hash(&mut hasher);
                AtomicU64::new(hasher.finish())
            },
        }
    }

    pub fn run(&self, f: impl (FnOnce() -> Box<dyn FnOnce() + Send + Sync>) + Send + Sync) {
        let mut cleanup = self.cleanup.lock().unwrap();
        cleanup.take().unwrap()();

        *cleanup = Some(f());
    }
}

impl Drop for Effect {
    fn drop(&mut self) {
        if let Some(cleanup) = self.cleanup.lock().unwrap().take() {
            cleanup();
        }
    }
}

#[derive(Default)]
pub struct EffectRegistry {
    effects: Mutex<Vec<Arc<Effect>>>,
}

impl EffectRegistry {
    pub fn register(&self, effect: Effect) {
        self.effects.lock().unwrap().push(Arc::new(effect));
    }

    pub fn get(&self, idx: usize) -> Option<Arc<Effect>> {
        self.effects.lock().unwrap().get(idx).cloned()
    }

    /// Returns true if new dependencies differ.
    pub fn update_deps<T: Hash>(&self, idx: usize, deps: T) -> bool {
        let mut hasher = DefaultHasher::new();
        deps.hash(&mut hasher);
        let hash = hasher.finish();

        let effect = self.get(idx).unwrap();

        let is_changed = hash != effect.hash.load(Ordering::Relaxed);

        if is_changed {
            effect.hash.store(hash, Ordering::Relaxed);
        }

        is_changed
    }

    pub fn clear(&self) {
        self.effects.lock().unwrap().clear();
    }
}
