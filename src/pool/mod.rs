//! This module contains definitions for creating [`ThreadPool`]s and
//! passing functions to the [`ThreadPool`].

//#![allow(unused_imports)]
use std::{
    io::{Error, ErrorKind},
    sync::{Arc, Mutex, mpsc::{channel, Receiver, Sender}},
    thread::{JoinHandle, spawn},
};

use crate::{unwrapmutex, unwrapreceiver, unwrapsender};

/// The typedef for a function sent to [`Worker`] threads to be run.
type Job = Box<dyn FnOnce() -> ConsolidatedMessage + Send + 'static>;

/// A message that is sent to [`Worker`] threads. This instructs each
/// [`Worker`] on what to do depending on which variant is sent.
/// 
/// # Variants
/// 
/// 1. Job([`Job`]) => A function to be sent to the [`Worker`] for running.
/// 2. Terminate => Tells the [`Worker`] to stop looping and join the main
/// thread.
pub enum WorkerMessage {
    Job(Job),
    Terminate,
}

/// Message to be sent to the [`ThreadPool`] holding the [`Worker`]s.
type ConsolidatedMessage = Result<(), Error>;

/// A [`ThreadPool`] stores [`Worker`]s who can run functions sent
/// using the [`ThreadPool::execute`] method. The [`ThreadPool`] is
/// responsible for delegating tasks to [`Worker`]s through a
/// [`std::sync::mpsc::Sender`] which transmits [`WorkerMessage`].
/// 
/// The [`ThreadPool`] is also responsible for telling each [`Worker`] to stop
/// running when it has been dropped to allow the program to shut down
/// gracefully.
pub struct ThreadPool {
    workers: Vec<Worker>,
    transmitter: Arc<Mutex<Sender<WorkerMessage>>>,
    receiver: Arc<Mutex<Receiver<ConsolidatedMessage>>>,
    received_ok: usize,
    received_err: usize,
}

impl ThreadPool {
    /// Creates a new [`ThreadPool`] instance. When you call this function,
    /// you have to specify the number of [`Worker`] threads that will be in
    /// the [`ThreadPool`], which must be at least 1.
    /// 
    /// # Parameters
    /// 1. ```threads: usize``` => Number of threads, must be at least 1.
    /// 
    /// # Error
    /// If `threads` is less than 1, a [`std::io::Error`] is returned.
    pub fn new(threads: usize) -> Result<Self, Error> {
        if threads < 1 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "You must have at least one thread to run the algorithm."
            ));
        }

        let (transmitter, worker_receiver) = channel::<WorkerMessage>();
        let (worker_transmitter, receiver) = channel::<ConsolidatedMessage>();
        let transmitter = Arc::new(Mutex::new(transmitter));
        let receiver = Arc::new(Mutex::new(receiver));
        let worker_receiver = Arc::new(Mutex::new(worker_receiver));
        let worker_transmitter = Arc::new(Mutex::new(worker_transmitter));
        let mut workers: Vec<Worker> = Vec::with_capacity(threads);
        for id in 0..threads {
            workers.push(Worker::new(
                id,
                worker_receiver.clone(),
                worker_transmitter.clone()
            ));
        }

        let received_ok: usize = 0;
        let received_err: usize = 0;

        return Ok(Self {
            workers,
            transmitter,
            receiver,
            received_ok,
            received_err,
        });
    }

    /// Clear the receiver and logs each [`Result`] to `self.received_ok` and
    /// `self.received_err`.
    fn read_receiver(&mut self) -> Result<(), Error> {
        while let Ok(message) = unwrapmutex!(self.receiver.lock()).try_recv() {
            if message.is_ok() {
                self.received_ok += 1;
            } else {
                self.received_err += 1;
            }
        }
        return Ok(());
    }

    /// Check how many jobs succeeded.
    pub fn jobs_ok(&mut self) -> Result<usize, Error> {
        self.read_receiver()?;
        return Ok(self.received_ok);
    }

    /// Check how many jobs has failed.
    pub fn jobs_err(&mut self) -> Result<usize, Error> {
        self.read_receiver()?;
        return Ok(self.received_err);
    }

    /// Resets `self.received_ok` and `self.received_err`.
    pub fn reset_log(&mut self) {
        self.received_ok = 0;
        self.received_err = 0;
    }

    /// Execute a function which runs once.
    pub fn execute<F>(&self, function: F) -> Result<(), Error>
    where
        F: FnOnce() -> ConsolidatedMessage + Send + 'static
    {
        let job = Box::new(function);
        return Ok(
            unwrapsender!(unwrapmutex!(self.transmitter.lock())
                .send(WorkerMessage::Job(job))
            )
        );
    }

    #[
        deprecated = "The result from each calculation will be directly sent \
        to another object."
    ]
    pub fn collect_node(&self) -> ConsolidatedMessage {
        return unwrapreceiver!(unwrapmutex!(self.receiver.lock()).recv());
    }
}

impl Drop for ThreadPool {
    /// Stops each [`Worker`] from running to safely shut down the
    /// [`ThreadPool`].
    fn drop(&mut self) {
        for _ in &self.workers {
            // It's safe to just unwrap like this here.
            self.transmitter
                .lock()
                .unwrap()
                .send(WorkerMessage::Terminate)
                .unwrap();
        }
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                // Double unwrap because the thread in Worker returns a Result.
                thread.join().unwrap();
            }
        }
    }
}

/// A [`Worker`] contains an `id` which identifies itself and has a `thread`
/// within itself.
struct Worker {
    pub id: usize,
    thread: Option<JoinHandle<()>>
}

impl Worker {
    /// Creates a new [`Worker`] instance.
    /// 
    /// # Parameters
    /// 
    /// 1. ```id: usize``` => Identifier for each [`Worker`]
    /// 2. ```receiver: Arc<Mutex<Receiver<WorkerMessage>>>``` => A receiver
    /// which receives instructions from the [`ThreadPool`] the [`Worker`]
    /// resides in.
    /// 3. ```transmitter: Arc<Mutex<Sender<ConsolidatedMessage>>>``` =>
    /// A transmitter to the [`ThreadPool`].
    pub fn new(
        id: usize,
        receiver: Arc<Mutex<Receiver<WorkerMessage>>>,
        transmitter: Arc<Mutex<Sender<ConsolidatedMessage>>>,
    ) -> Self {
        let thread = spawn(move || loop {
            let message = receiver
                .lock()
                .unwrap()
                .recv()
                .unwrap();

            match message {
                WorkerMessage::Job(job) => {
                    transmitter
                        .lock()
                        .unwrap()
                        .send(job())
                        .unwrap();
                },
                WorkerMessage::Terminate => {
                    return ();
                }
            }
        });

        return Self {id, thread: Some(thread)};
    }
}