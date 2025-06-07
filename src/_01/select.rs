use std::{pin::Pin, time::Duration};

use futures::{StreamExt, stream::FuturesUnordered};
use tokio::sync::mpsc;

pub struct RandomSelectActor {
    pub incoming_tasks: mpsc::Receiver<u64>,
    pub processing_tasks: FuturesUnordered<Pin<Box<dyn Future<Output = u64> + Send>>>,
    pub results: mpsc::Sender<u64>,
}

impl RandomSelectActor {
    pub async fn run(mut self) {
        loop {
            tokio::select! {
                task = self.incoming_tasks.recv() => {
                    match task {
                        Some(task) => {
                            self.processing_tasks.push(Box::pin(async move {
                                tokio::time::sleep(Duration::from_micros(10)).await;
                                task * 2
                            }));
                        }
                        None => {
                            return;
                        }
                    }
                }
                Some(result) = self.processing_tasks.next() => {
                    self.results.send(result).await.unwrap();
                }
            }
        }
    }
}
