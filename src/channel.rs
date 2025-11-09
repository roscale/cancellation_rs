use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex},
};

use crate::stop_source::StopToken;

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let channel = Arc::new(Channel::<T>::new());
    (
        Sender {
            channel: channel.clone(),
        },
        Receiver { channel },
    )
}

pub struct Channel<T> {
    queue: Mutex<VecDeque<T>>,
    cond_var: Arc<Condvar>,
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn send(&self, item: T) {
        let mut guard = self.queue.lock().unwrap();
        guard.push_back(item);
        self.cond_var.notify_one();
    }

    pub fn receive(&self, stop_token: &StopToken) -> Option<T> {
        let guard = self.queue.lock().unwrap();

        stop_token.register_cond_var(self.cond_var.clone());

        let mut guard = self
            .cond_var
            .wait_while(guard, |vec| vec.is_empty() && !stop_token.stop_requested())
            .unwrap();

        stop_token.unregister_cond_var();

        guard.pop_front()
    }
}

impl<T> Default for Channel<T> {
    fn default() -> Self {
        Self {
            queue: Default::default(),
            cond_var: Default::default(),
        }
    }
}

#[derive(Clone)]
pub struct Sender<T> {
    channel: Arc<Channel<T>>,
}

impl<T> Sender<T> {
    pub fn send(&self, item: T) {
        self.channel.send(item);
    }
}

pub struct Receiver<T> {
    channel: Arc<Channel<T>>,
}

impl<T> Receiver<T> {
    pub fn receive(&self, stop_token: &StopToken) -> Option<T> {
        self.channel.receive(stop_token)
    }
}
