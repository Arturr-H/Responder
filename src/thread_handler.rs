/*- Imports -*/
use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

/*- Structs, enums & unions -*/
#[allow(dead_code)]
pub struct MainThreadHandler {
    threads: Vec<Worker>,
    sender: mpsc::Sender<Task>,
}

/*- Handles a connection -*/
#[allow(dead_code)]
pub struct Worker {
    thread: JoinHandle<()>,
}

/*- The tasks which Workers will do -*/
pub type Task = Box<dyn FnOnce() + Send + 'static>;

/*- Method implementations -*/
impl MainThreadHandler {
    pub fn new(num_threads: u16) -> Self {
        /*- Needs to be bigger than 0 -*/
        if num_threads < 1 {
            panic!("Number of threads must be bigger than 0");
        };

        /*- Open mpsc channel -*/
        let (sender, rcv): (Sender<Task>, Receiver<Task>) = mpsc::channel::<Task>();
        let rcv = Arc::new(Mutex::new(rcv));

        /*- Initialize threads and allocate the right amount of memory -*/
        let mut threads: Vec<Worker> = Vec::with_capacity(num_threads as usize);
        for _ in 0..num_threads {
            threads.push(Worker::new(Arc::clone(&rcv)));
        }

        /*- Return -*/
        MainThreadHandler { threads, sender }
    }

    pub fn exec<T>(&self, t: T)
    where
        T: FnOnce() + Send + 'static,
    {
        let task = Box::new(t);

        /*- Send the job down the channel -*/
        self.sender.send(task).unwrap_or(());
    }
}
impl Worker {
    pub fn new(reciever: Arc<Mutex<Receiver<Task>>>) -> Self {
        let thread = thread::spawn(move || loop {
            /*- Get the task -*/
            let task = match match reciever.lock() {
                Ok(v) => v,
                Err(_) => return,
            }
            .recv()
            {
                Ok(v) => v,
                Err(_) => return,
            };

            /*- Execute task -*/
            task();
        });

        /*- Return -*/
        Worker { thread }
    }
}
