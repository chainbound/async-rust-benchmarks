use futures::{StreamExt, stream::FuturesUnordered};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use super::{Actor, ActorMetrics, TASK_DURATION, Task};

pub struct RandomSelectActor {
    pub incoming_tasks: mpsc::Receiver<Instant>,
    pub processing_tasks: FuturesUnordered<Task>,
    pub results: mpsc::Sender<Duration>,
    pub metrics: ActorMetrics,
}

impl Actor for RandomSelectActor {
    async fn run(mut self) -> ActorMetrics {
        loop {
            tokio::select! {
                task = self.incoming_tasks.recv() => {
                    match task {
                        Some(task) => {
                            self.processing_tasks.push(Task::new(task, TASK_DURATION));
                            self.metrics.max_pending_tasks = self
                                .processing_tasks
                                .len()
                                .max(self.metrics.max_pending_tasks);
                        }
                        None => {
                            return self.metrics;
                        }
                    }
                }

                Some(result) = self.processing_tasks.next() => {
                    // Offset the result by the task duration to get the actual latency.
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
    pub metrics: ActorMetrics,
}

impl Actor for BiasedSelectActor {
    async fn run(mut self) -> ActorMetrics {
        loop {
            tokio::select! {
                // Interestingly, biasing by prioritizing incoming work over processing local work is faster than anything else.
                // But it would also incur the highest memory usage.
                biased;

                Some(result) = self.processing_tasks.next() => {
                    // Offset the result by the task duration to get the actual latency.
                    self.results.try_send(Instant::now().duration_since(result) - TASK_DURATION).unwrap();
                }

                task = self.incoming_tasks.recv() => {
                    match task {
                        Some(task) => {
                            self.processing_tasks.push(Task::new(task, TASK_DURATION));
                            self.metrics.max_pending_tasks = self
                                .processing_tasks
                                .len()
                                .max(self.metrics.max_pending_tasks);
                        }
                        None => {
                            return self.metrics;
                        }
                    }
                }
            }
        }
    }
}
