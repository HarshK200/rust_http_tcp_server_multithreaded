use std::{
    sync::{mpsc, Arc, Mutex},
    thread::JoinHandle,
};

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}
impl Worker {
    pub fn new(id: usize, reciver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Result<Worker, String> {
        let thread_builder = std::thread::Builder::new();

        // NOTE: the move || loop {} just runs infinitly because of the loop keyword
        let thread_creation_result = thread_builder.spawn(move || loop {
            let job = reciver.lock().unwrap().recv();

            match job {
                Ok(job) => {
                    job();
                    println!("thread {id} finished executing the job");
                }
                Err(_) => {
                    println!("worker {id} disconnected. shutting down...");
                    break;
                }
            }
        });

        match thread_creation_result {
            Ok(t) => {
                return Ok(Worker {
                    id,
                    thread: Some(t),
                });
            }
            Err(e) => {
                return Err(format!("error occured during thread creating: {}", e));
            }
        };
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Result<ThreadPool, String> {
        if size <= 0 {
            return Err(String::from(
                "error the no. of thread cannot be negative or 0",
            ));
        }

        let (sender, reciver) = mpsc::channel();
        let arc_mtx_reciver = Arc::new(Mutex::new(reciver));
        let mut workers = Vec::with_capacity(size);

        for i in 0..size {
            // NOTE: creating worker threads
            let worker = Worker::new(i, Arc::clone(&arc_mtx_reciver));
            match worker {
                Ok(w) => workers.push(w),
                Err(e) => {
                    return Err(e);
                }
            }
        }

        return Ok(ThreadPool {
            workers,
            sender: Some(sender),
        });
    }

    pub fn execute<F>(&self, closure: F) -> Result<(), String>
    where
        F: FnOnce() -> () + Send + 'static,
    {
        let job = Box::new(closure);

        let result = self.sender.as_ref().unwrap().send(job);

        match result {
            Ok(_) => return Ok(()),
            Err(e) => {
                return Err(format!(
                    "error occured during sending the job\nerror: {}",
                    e
                ));
            }
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // NOTE: droping the sender so that the channel closes and the worker threads stops waiting
        // for messages/jobs since recv will return an error
        drop(self.sender.take());

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
            println!("worker id: {} shutting down gracefully...", worker.id);
        }
    }
}
