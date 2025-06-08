use futures::{StreamExt, stream::FuturesUnordered};
use std::time::Duration;
use tokio::sync::mpsc;

use super::Task;

pub struct RandomSelectActor {
    pub incoming_tasks: mpsc::Receiver<u64>,
    pub processing_tasks: FuturesUnordered<Task>,
    pub results: mpsc::Sender<u64>,
}

impl RandomSelectActor {
    pub async fn run(mut self) {
        loop {
            tokio::select! {
                task = self.incoming_tasks.recv() => {
                    match task {
                        Some(task) => {
                            self.processing_tasks.push(Task::new(task, Duration::from_micros(10)));
                        }
                        None => {
                            return;
                        }
                    }
                }

                Some(result) = self.processing_tasks.next() => {
                    self.results.try_send(result).unwrap();
                }
            }
        }
    }
}

pub struct BiasedSelectActor {
    pub incoming_tasks: mpsc::Receiver<u64>,
    pub processing_tasks: FuturesUnordered<Task>,
    pub results: mpsc::Sender<u64>,
}

impl BiasedSelectActor {
    pub async fn run(mut self) {
        loop {
            tokio::select! {
                // Interestingly, biasing by prioritizing incoming work over processing local work is faster than anything else.
                // But it would also incur the highest memory usage.
                biased;

                Some(result) = self.processing_tasks.next() => {
                    self.results.try_send(result).unwrap();
                }

                task = self.incoming_tasks.recv() => {
                    match task {
                        Some(task) => {
                            self.processing_tasks.push(Task::new(task, Duration::from_micros(10)));
                        }
                        None => {
                            return;
                        }
                    }
                }
            }
        }
    }
}
