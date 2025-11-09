use std::sync::{
    Arc, Condvar, Mutex,
    atomic::{AtomicBool, Ordering},
};

#[derive(Default, Clone)]
pub struct StopSource {
    data: Arc<StopState>,
}

#[derive(Clone)]
pub struct StopToken {
    data: Arc<StopState>,
}

#[derive(Default)]
struct StopState {
    stop_requested: AtomicBool,
    cond_var: Mutex<Option<Arc<Condvar>>>,
}

impl StopSource {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn request_stop(&self) {
        self.data.stop_requested.store(true, Ordering::SeqCst);

        let guard = self.data.cond_var.lock().unwrap();
        if let Some(cond_var) = &*guard {
            cond_var.notify_one();
        }
    }

    pub fn stop_token(&self) -> StopToken {
        StopToken {
            data: self.data.clone(),
        }
    }
}

impl StopToken {
    pub fn stop_requested(&self) -> bool {
        self.data.stop_requested.load(Ordering::SeqCst)
    }

    pub fn register_cond_var(&self, cond_var: Arc<Condvar>) {
        *self.data.cond_var.lock().unwrap() = Some(cond_var);
    }

    pub fn unregister_cond_var(&self) {
        *self.data.cond_var.lock().unwrap() = None;
    }
}
