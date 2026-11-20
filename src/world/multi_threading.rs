use std::{
    sync::{Arc, Mutex, mpsc},
    thread::{self, JoinHandle},
};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<JoinHandle<()>>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    pub fn new(thread_count: usize) -> Self {
        assert!(thread_count > 0);

        let (sender, receiver) = mpsc::channel::<Job>();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(thread_count);

        for worker_id in 0..thread_count {
            let receiver = Arc::clone(&receiver);

            let worker = thread::Builder::new()
                .name(format!("chunk-worker-{worker_id}"))
                .spawn(move || {
                    loop {
                        let job = receiver
                            .lock()
                            .expect("chunk worker receiver was poisoned")
                            .recv();

                        match job {
                            Ok(job) => job(),
                            Err(_) => break,
                        }
                    }
                })
                .expect("failed to start chunk worker");
            workers.push(worker);
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F: FnOnce() + Send + 'static>(&self, job: F) {
        self.sender
            .as_ref()
            .expect("chunk worker pool is shutting down")
            .send(Box::new(job))
            .expect("chunk worker pool has stopped");
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}
