use std::thread::JoinHandle;

use crate::stop_source::{StopSource, StopToken};

pub fn spawn<T, F>(f: F) -> JThread<T>
where
    T: Send + 'static,
    F: FnOnce(StopToken) -> T + Send + 'static,
{
    let stop_source = StopSource::new();
    let stop_token = stop_source.stop_token();

    JThread {
        join_handle: Some(std::thread::spawn(|| f(stop_token))),
        stop_source,
    }
}

pub fn spawn_with_stop_source<T, F>(f: F, stop_source: StopSource) -> JThread<T>
where
    T: Send + 'static,
    F: FnOnce(StopToken) -> T + Send + 'static,
{
    let stop_token = stop_source.stop_token();

    JThread {
        join_handle: Some(std::thread::spawn(|| f(stop_token))),
        stop_source,
    }
}

pub struct JThread<T>
where
    T: Send + 'static,
{
    join_handle: Option<JoinHandle<T>>,
    stop_source: StopSource,
}

impl<T> JThread<T>
where
    T: Send + 'static,
{
    pub fn request_stop(&mut self) {
        self.stop_source.request_stop();
    }

    pub fn join(mut self) -> T {
        self.join_handle.take().unwrap().join().unwrap()
    }
}

impl<T> Drop for JThread<T>
where
    T: Send + 'static,
{
    fn drop(&mut self) {
        self.request_stop();

        if let Some(join_handle) = self.join_handle.take() {
            let _ = join_handle.join().unwrap();
        }
    }
}
