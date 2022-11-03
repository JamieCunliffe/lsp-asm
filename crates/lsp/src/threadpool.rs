use std::panic::AssertUnwindSafe;

use crossbeam_channel::{Receiver, Sender};

pub trait Task {
    fn process(self);
}

pub struct ThreadPool<T: Task + Send> {
    send: Sender<T>,
}

impl<T: Task + Send + 'static> ThreadPool<T> {
    pub fn new(num_threads: usize) -> Self {
        let (send, recv) = crossbeam_channel::unbounded();
        for _ in 0..num_threads {
            let recv = recv.clone();
            std::thread::spawn(move || Self::process(recv));
        }
        Self { send }
    }

    pub fn add_task(&self, task: T) {
        let _ = self.send.send(task);
    }

    fn process(recv: Receiver<T>) {
        while let Ok(msg) = recv.recv() {
            let _ = std::panic::catch_unwind(AssertUnwindSafe(|| msg.process()));
        }
    }
}
