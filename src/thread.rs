use std::sync::{Arc, mpsc, Mutex};
use std::thread;

pub struct ThreadPool { 
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for _ in 0..size {
            workers.push(Worker::new(Arc::clone(&receiver)));
        }

        Self { workers, sender: Some(sender) }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || loop {
            match receiver.lock().unwrap().recv() {
                Ok(job) => job(),
                Err(_) => break,
            };
        });

        Self { thread: Some(thread) }
    }
}
