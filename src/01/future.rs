use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use futures::{StreamExt, stream::FuturesUnordered};
use tokio::{sync::mpsc, task::coop::unconstrained};

use super::{Actor, ActorMetrics, TASK_DURATION, Task};

/// A simple actor that implements the [`Future`] trait.
/// It will receive tasks from a buffered channel and process them in parallel.
/// The task consists of multiplying a number by 2, with an artificial async delay of 1ms per task.
///
/// # Poll order
/// 1. Continue work in progress
/// 2. Send finished results on the results channel
/// 3. Receive new tasks from the incoming channel
pub struct FutureActor<T> {
    pub incoming_tasks: mpsc::Receiver<Instant>,
    pub processing_tasks: FuturesUnordered<Task>,
    pub results: mpsc::Sender<Duration>,
    pub metrics: ActorMetrics,
    pub _unconstrained: PhantomData<T>,
}

impl<T> FutureActor<T> {
    pub fn new(incoming_tasks: mpsc::Receiver<Instant>, results: mpsc::Sender<Duration>) -> Self {
        Self {
            incoming_tasks,
            processing_tasks: FuturesUnordered::new(),
            results,
            metrics: ActorMetrics::new(),
            _unconstrained: PhantomData,
        }
    }
}

#[derive(Default)]
pub struct Constrained;

#[derive(Default)]
pub struct Unconstrained;

impl Actor for FutureActor<Constrained> {
    fn run(self) -> impl Future<Output = ActorMetrics> + Send + 'static {
        self
    }
}

impl Actor for FutureActor<Unconstrained> {
    fn run(self) -> impl Future<Output = ActorMetrics> + Send + 'static {
        unconstrained(self)
    }
}

impl<T> Future for FutureActor<T>
where
    T: Unpin,
{
    type Output = ActorMetrics;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        loop {
            if let Poll::Ready(Some(result)) = this.processing_tasks.poll_next_unpin(cx) {
                // Offset the result by the task duration to get the actual latency.
                this.results
                    .try_send(Instant::now().duration_since(result) - TASK_DURATION)
                    .unwrap();

                continue;
            }

            match this.incoming_tasks.poll_recv(cx) {
                Poll::Ready(Some(task)) => {
                    this.processing_tasks.push(Task::new(task, TASK_DURATION));
                    this.metrics.max_pending_tasks = this
                        .processing_tasks
                        .len()
                        .max(this.metrics.max_pending_tasks);

                    continue;
                }
                Poll::Ready(None) => {
                    return Poll::Ready(this.metrics.clone());
                }
                Poll::Pending => {}
            }

            return Poll::Pending;
        }
    }
}
