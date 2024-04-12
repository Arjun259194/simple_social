use std::{
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
};

pub mod server;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Messages>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        let (s, r) = mpsc::channel();

        let mut workers = Vec::with_capacity(size);

        let receiver = Arc::new(Mutex::new(r));

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender: s }
    }

    pub fn execute<F>(&self, f: F) where F: FnOnce() + Send + 'static {
        let job = Messages::NewJob(Box::new(f));
        self.sender.send(job).unwrap();
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Messages {
    NewJob(Job),
    Terminate,
}

struct Worker {
    id: usize,
    thread: JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Messages>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();
            match message {
                Messages::NewJob(job) => job(),
                Messages::Terminate => {
                    println!("Terminating thread {id}");
                    break;
                }
            }
        });

        Worker { id, thread }
    }
}
