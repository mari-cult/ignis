//! [Secure Computing Mode](https://en.wikipedia.org/wiki/Seccomp).
//!
//! # References
//!
//! - [`include/uapi/linux/seccomp.h`](https://github.com/torvalds/linux/blob/master/include/uapi/linux/seccomp.h)
//! - [`seccomp_unotify(2)`](https://man7.org/linux/man-pages/man2/seccomp_unotify.2.html)

use {
    crate::syscall::{Error, Id},
    std::{fmt, io, os::fd::OwnedFd},
};

pub mod bpf;

mod c;

/// System call notification.
#[doc(alias = "seccomp_notif")]
pub struct Notification<'listener> {
    listener: &'listener mut Listener,
    notification: c::seccomp_notif,
}

/// SecComp userspace listener.
#[derive(Debug)]
pub struct Listener {
    fd: OwnedFd,
}

impl Listener {
    /// Install the specified BPF program, and return a listener.
    pub fn install(program: &bpf::Program<'_>) -> io::Result<Listener> {
        c::seccomp_new_listener(program).map(|fd| Listener { fd })
    }

    /// Blocks until the installed BPF program triggers [`Action::UserNotify`][bpf::Action::UserNotify].
    pub fn recv(&mut self) -> io::Result<Notification<'_>> {
        let notification = c::seccomp_ioctl_notif_recv(&self.fd)?;

        Ok(Notification {
            listener: self,
            notification,
        })
    }
}

impl<'listener> Notification<'listener> {
    /// Arguments provided to the system call.
    pub fn args(&self) -> [u64; 6] {
        self.notification.data.args
    }

    /// Return the system call ID.
    pub fn syscall(&self) -> Result<Id, u32> {
        Id::from_raw(self.notification.data.nr).ok_or(self.notification.data.nr)
    }

    /// ID of the process which produced this notification.
    pub fn process_id(&self) -> u32 {
        self.notification.pid
    }

    /// Reply to the notification to continue as normal.
    ///
    /// # Safety
    ///
    /// As this mechanism is racy, care must be taken to ensure
    /// a system call isn't overwritten by the traced thread.
    pub unsafe fn send_continue(&mut self) -> io::Result<()> {
        self.send_response(c::seccomp_notif_resp {
            id: self.notification.id,
            flags: c::SECCOMP_USER_NOTIF_FLAG_CONTINUE,
            ..Default::default()
        })
    }

    /// Reply to the notification with an error.
    pub fn send_error(&mut self, error: Error) -> io::Result<()> {
        self.send_response(c::seccomp_notif_resp {
            id: self.notification.id,
            error: -error.raw_os_error(),
            ..Default::default()
        })
    }

    /// Reply to the notification.
    fn send_response(&mut self, response: c::seccomp_notif_resp) -> io::Result<()> {
        c::seccomp_ioctl_notif_send(&self.listener.fd, response)
    }
}

impl<'listener> fmt::Debug for Notification<'listener> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = fmt.debug_struct("Notification");

        debug.field("process_id", &self.process_id());

        match self.syscall() {
            Ok(syscall) => debug.field("syscall", &syscall),
            Err(syscall) => debug.field("syscall", &syscall),
        };

        debug.field("args", &self.args()).finish_non_exhaustive()
    }
}
