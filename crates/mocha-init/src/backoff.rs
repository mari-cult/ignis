use {
    std::{
        future::{self, Future},
        pin::Pin,
        task::{Context, Poll},
        time::Duration,
    },
    tokio::time::{self, Instant, Sleep},
};

pub struct Backoff {
    duration: Duration,
    delay: Pin<Box<Sleep>>,
}

impl Backoff {
    /// Create a `Backoff`, with the specified initial duration.
    pub fn new(duration: Duration) -> Self {
        let delay = Box::pin(time::sleep(duration));

        Self { duration, delay }
    }

    /// Completes when the next instant in the backoff has been reached.
    pub async fn tick(&mut self) -> Instant {
        future::poll_fn(|context| self.poll_tick(context)).await
    }

    /// Polls for the next instant in the backoff to be reached.
    pub fn poll_tick(&mut self, context: &mut Context<'_>) -> Poll<Instant> {
        match Pin::new(&mut self.delay).poll(context) {
            Poll::Pending => return Poll::Pending,
            _ => {}
        }

        let timeout = self.delay.deadline();

        self.duration = self.duration * 2;
        self.delay = Box::pin(time::sleep(self.duration));

        Poll::Ready(timeout)
    }
}
