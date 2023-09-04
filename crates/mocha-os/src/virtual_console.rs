use std::{fs::File, io, num::NonZeroU8, os::unix::io::OwnedFd, process::Stdio};

/// A virtual console (`/dev/tty{index}`).
pub struct VirtualConsole {
    fd: OwnedFd,
}

impl VirtualConsole {
    /// Open the specified virtual console.
    pub fn open(index: NonZeroU8) -> io::Result<Self> {
        let path = format!("/dev/tty{index}");
        let fd = File::options().read(true).write(true).open(path)?.into();

        Ok(Self { fd })
    }

    /// Creates a new `VirtualConsole` that shares the same underlying
    /// file descriptor as the existing `VirtualConsole`.
    pub fn try_clone(&self) -> io::Result<Self> {
        let fd = self.fd.try_clone()?;

        Ok(Self { fd })
    }

    /// Creates three new `VirtualConsole`s that share the underlying
    /// file descriptor as the existing `VirtualConsole`.
    pub fn clone_for_stdio(&self) -> io::Result<(Self, Self, Self)> {
        let stdin = self.try_clone()?;
        let stdout = self.try_clone()?;
        let stderr = self.try_clone()?;

        Ok((stdin, stdout, stderr))
    }
}

impl From<VirtualConsole> for Stdio {
    fn from(console: VirtualConsole) -> Self {
        console.fd.into()
    }
}
