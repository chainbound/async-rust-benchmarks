use pin_project_lite::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio::time::Sleep;

pub mod future;
pub mod select;

// We use `pin_project` here because `Sleep` is not `Unpin`. This means that the only way to
// use it in a `Future` is to put it on the heap with `Box::pin`, which we want to avoid.
pin_project! {
    pub struct Task {
        #[pin]
        sleep: Sleep,
        value: u64,
    }
}

impl Task {
    fn new(value: u64, duration: Duration) -> Self {
        Self {
            sleep: tokio::time::sleep(duration),
            value,
        }
    }
}

impl Future for Task {
    type Output = u64;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.sleep.poll(cx) {
            Poll::Ready(()) => Poll::Ready(*this.value * 2),
            Poll::Pending => Poll::Pending,
        }
    }
}
