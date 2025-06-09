use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use futures::{StreamExt, stream::FuturesUnordered};
use tokio::sync::mpsc;

use super::{TASK_DURATION, Task};

/// A simple actor that implements the [`Future`] trait.
/// It will receive tasks from a buffered channel and process them in parallel.
/// The task consists of multiplying a number by 2, with an artificial async delay of 1ms per task.
///
/// # Poll order
/// 1. Continue work in progress
/// 2. Send finished results on the results channel
/// 3. Receive new tasks from the incoming channel
pub struct FutureActor {
    pub incoming_tasks: mpsc::Receiver<Instant>,
    pub processing_tasks: FuturesUnordered<Task>,
    pub results: mpsc::Sender<Duration>,
}

impl Future for FutureActor {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        loop {
            if let Poll::Ready(Some(result)) = this.processing_tasks.poll_next_unpin(cx) {
                this.results
                    .try_send(Instant::now().duration_since(result) - TASK_DURATION)
                    .unwrap();
                continue;
            }

            match this.incoming_tasks.poll_recv(cx) {
                Poll::Ready(Some(task)) => {
                    this.processing_tasks.push(Task::new(task, TASK_DURATION));

                    continue;
                }
                Poll::Ready(None) => {
                    return Poll::Ready(());
                }
                Poll::Pending => {}
            }

            return Poll::Pending;
        }
    }
}
