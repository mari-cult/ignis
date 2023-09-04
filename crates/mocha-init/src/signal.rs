use {
    std::{
        future, io,
        task::{Context, Poll},
    },
    tokio::signal::unix::{self, SignalKind},
};

macro_rules! poll_signal {
    ($listener:expr, $context:expr, $signal:expr) => {
        match $listener.poll_recv($context) {
            Poll::Ready(Some(())) => return Poll::Ready($signal),
            _ => {}
        }
    };
}
/// Which signal was received.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Signal {
    Child,
    UserDefined1,
    UserDefined2,
}

/// A signal listener.
#[derive(Debug)]
pub struct Signals {
    child: unix::Signal,
    user_defined1: unix::Signal,
    user_defined2: unix::Signal,
}

impl Signals {
    pub fn new() -> io::Result<Self> {
        let child = unix::signal(SignalKind::child())?;
        let user_defined1 = unix::signal(SignalKind::user_defined1())?;
        let user_defined2 = unix::signal(SignalKind::user_defined2())?;

        Ok(Self {
            child,
            user_defined1,
            user_defined2,
        })
    }

    /// Receive the next signal notification.
    pub async fn next(&mut self) -> Signal {
        future::poll_fn(|context| self.poll_next(context)).await
    }

    /// Poll to receive the next signal notification.
    pub fn poll_next(&mut self, context: &mut Context<'_>) -> Poll<Signal> {
        poll_signal!(self.child, context, Signal::Child);
        poll_signal!(self.user_defined1, context, Signal::UserDefined1);
        poll_signal!(self.user_defined2, context, Signal::UserDefined2);

        Poll::Pending
    }
}
