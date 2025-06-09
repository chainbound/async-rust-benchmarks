use futures::{StreamExt, stream::FuturesUnordered};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use super::{TASK_DURATION, Task};

pub struct RandomSelectActor {
    pub incoming_tasks: mpsc::Receiver<Instant>,
    pub processing_tasks: FuturesUnordered<Task>,
    pub results: mpsc::Sender<Duration>,
}

impl RandomSelectActor {
    pub async fn run(mut self) {
        loop {
            tokio::select! {
                task = self.incoming_tasks.recv() => {
                    match task {
                        Some(task) => {
                            self.processing_tasks.push(Task::new(task, TASK_DURATION));
                        }
                        None => {
                            return;
                        }
                    }
                }

                Some(result) = self.processing_tasks.next() => {
                    self.results.try_send(Instant::now().duration_since(result) - TASK_DURATION).unwrap();
                }
            }
        }
    }
}

pub struct BiasedSelectActor {
    pub incoming_tasks: mpsc::Receiver<Instant>,
    pub processing_tasks: FuturesUnordered<Task>,
    pub results: mpsc::Sender<Duration>,
}

impl BiasedSelectActor {
    pub async fn run(mut self) {
        loop {
            tokio::select! {
                // Interestingly, biasing by prioritizing incoming work over processing local work is faster than anything else.
                // But it would also incur the highest memory usage.
                biased;

                Some(result) = self.processing_tasks.next() => {
                    self.results.try_send(Instant::now().duration_since(result) - TASK_DURATION).unwrap();
                }

                task = self.incoming_tasks.recv() => {
                    match task {
                        Some(task) => {
                            self.processing_tasks.push(Task::new(task, TASK_DURATION));
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
