use std::{
    any::Any,
    sync::{Arc, Mutex, atomic::AtomicBool},
};

pub trait StateValue: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn eq(&self, other: &dyn StateValue) -> bool;
}

impl<T: Send + Sync + PartialEq + 'static> StateValue for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn eq(&self, other: &dyn StateValue) -> bool {
        if let Some(other_t) = other.as_any().downcast_ref::<T>() {
            self == other_t
        } else {
            false
        }
    }
}

#[derive(Default)]
pub struct StateRegistry {
    state: Mutex<Vec<Arc<dyn StateValue>>>,
    is_dirty: AtomicBool,
}

impl StateRegistry {
    pub fn register(&self, value: impl StateValue + 'static) -> Arc<dyn StateValue> {
        let value: Arc<dyn StateValue> = Arc::new(value);
        self.state.lock().unwrap().push(Arc::clone(&value));
        value
    }

    pub fn get<T: Send + Sync + PartialEq + 'static>(
        &self,
        idx: usize,
    ) -> Option<Arc<dyn StateValue>> {
        self.state.lock().unwrap().get(idx).cloned()
    }

    pub fn update<T: Send + Sync + PartialEq + 'static>(&self, idx: usize, value: T) -> bool {
        let mut state = self.state.lock().unwrap();
        let prev_value = state.get_mut(idx).unwrap();
        if prev_value.eq(&value) {
            return false;
        }

        self.is_dirty
            .store(true, std::sync::atomic::Ordering::Relaxed);
        *prev_value = Arc::new(value);

        true
    }

    pub fn mark_clean(&self) {
        self.is_dirty
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }
}
