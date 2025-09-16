use std::{
    any::Any,
    sync::{Arc, Mutex, atomic::AtomicUsize},
};

pub trait StateValue: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn eq(&self, other: &dyn StateValue) -> bool;
}

impl<T: Send + Sync + PartialEq + Clone + 'static> StateValue for T {
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
pub struct ViewContextState {
    state: Mutex<Vec<Arc<dyn StateValue>>>,
    is_dirty: Mutex<bool>,
    current_order: AtomicUsize,
}

impl ViewContextState {
    pub fn insert(&self, value: impl StateValue + 'static) {
        self.state.lock().unwrap().push(Arc::new(value));
    }

    pub fn get<T: Send + Sync + PartialEq + Clone + 'static>(&self, idx: usize) -> Option<T> {
        self.state
            .lock()
            .unwrap()
            .get(idx)
            .and_then(|val| val.as_any().downcast_ref::<T>())
            .cloned()
    }

    pub fn set<T: Send + Sync + PartialEq + Clone + 'static>(&self, idx: usize, value: T) {
        let mut state = self.state.lock().unwrap();
        let prev_value = state.get_mut(idx).unwrap();
        if prev_value.eq(&value) {
            return;
        }

        *self.is_dirty.lock().unwrap() = true;
        *prev_value = Arc::new(value);
    }

    pub fn reset_order(&self) {
        self.current_order
            .store(0, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn get_order(&self) -> usize {
        self.current_order
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    pub fn is_dirty(&self) -> bool {
        *self.is_dirty.lock().unwrap()
    }
}
