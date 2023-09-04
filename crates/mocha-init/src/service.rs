use {
    super::backoff::Backoff,
    std::{
        io,
        process::ExitStatus,
        task::{Context, Poll},
        time::Duration,
    },
    tokio::process::{Child, Command},
};

pub struct Service {
    new_command: Box<dyn FnMut() -> io::Result<Command> + Send + Sync + 'static>,
    backoff: Backoff,
    child: Option<Child>,
}

impl Service {
    pub fn poll(&mut self, context: &mut Context<'_>) -> Poll<io::Result<ExitStatus>> {
        if let Some(child) = &mut self.child {
            let poll = match child.try_wait() {
                Ok(Some(status)) => Poll::Ready(Ok(status)),
                Ok(None) => return Poll::Pending,
                Err(error) => Poll::Ready(Err(error)),
            };

            self.child = None;

            poll
        } else {
            if self.backoff.poll_tick(context).is_ready() {
                self.child = Some((self.new_command)()?.spawn()?);
            }

            Poll::Pending
        }
    }
}

/// Spawn a service using the command provided by the closure.
pub fn spawn<F: FnMut() -> io::Result<Command> + Send + Sync + 'static>(
    new_command: F,
) -> io::Result<Service> {
    let mut new_command = Box::new(new_command);
    let child = Some((new_command)()?.spawn()?);
    let backoff = Backoff::new(Duration::from_secs(1));

    Ok(Service {
        new_command,
        backoff,
        child,
    })
}
