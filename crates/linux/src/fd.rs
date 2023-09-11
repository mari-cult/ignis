//! File-descriptor related functionality.

use std::{
    io::{self, IoSlice, IoSliceMut},
    os::unix::{
        io::{AsFd, AsRawFd, FromRawFd, OwnedFd},
        net::{AncillaryData, SocketAncillary, UnixStream},
    },
    slice,
};

/// A file descriptor sender.
///
/// A wrapper over [`UnixStream`](UnixStream), that transfers file descriptors between processes.
pub struct FdSender {
    stream: UnixStream,
}

/// A file descriptor sender.
///
/// A wrapper over [`UnixStream`](UnixStream), that transfers file descriptors between processes.
pub struct FdReceiver {
    stream: UnixStream,
}

impl FdSender {
    /// Send a file descriptor.
    pub fn send<Fd: AsFd>(&self, fd: Fd) -> io::Result<()> {
        // At least one byte must be sent.
        let buf = [0u8; 1];
        let buf = IoSlice::new(&buf);
        let bufs = slice::from_ref(&buf);

        let mut buf = [0; 128];
        let mut ancillary = SocketAncillary::new(&mut buf);

        let fd = fd.as_fd().as_raw_fd();
        let fds = slice::from_ref(&fd);

        ancillary.add_fds(fds);
        self.stream
            .send_vectored_with_ancillary(bufs, &mut ancillary)?;

        Ok(())
    }
}

impl FdReceiver {
    /// Receive a file descriptor.
    pub fn recv(&self) -> io::Result<OwnedFd> {
        // At least one byte must be received.
        let mut buf = [0u8; 1];
        let mut buf = IoSliceMut::new(&mut buf);
        let bufs = slice::from_mut(&mut buf);

        let mut buf = [0; 128];
        let mut ancillary = SocketAncillary::new(&mut buf);

        self.stream
            .recv_vectored_with_ancillary(bufs, &mut ancillary)?;

        let Some(result) = ancillary.messages().next() else {
            return Err(io_other("no ancillary messages were sent"));
        };

        let Ok(message) = result else {
            return Err(io_other(
                "encountered a problem while receiving ancillary message",
            ));
        };

        let AncillaryData::ScmRights(mut rights) = message else {
            return Err(io_other("expected ancillary rights"));
        };

        let Some(fd) = rights.next() else {
            return Err(io_other("expected a file descriptor"));
        };

        Ok(unsafe { OwnedFd::from_raw_fd(fd) })
    }
}

/// Create a new file descriptor channel.
pub fn channel() -> io::Result<(FdSender, FdReceiver)> {
    UnixStream::pair().map(|(sender, receiver)| {
        let sender = FdSender { stream: sender };
        let receiver = FdReceiver { stream: receiver };

        (sender, receiver)
    })
}

// Create an other I/O error with a the specified message.
fn io_other(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::Other, message)
}
