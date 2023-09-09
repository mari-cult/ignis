//! [Secure Computing Mode](https://en.wikipedia.org/wiki/Seccomp).
//!
//! # References
//!
//! - [`include/uapi/linux/seccomp.h`](https://github.com/torvalds/linux/blob/master/include/uapi/linux/seccomp.h)

use {
    crate::syscall::syscall,
    std::{
        io,
        mem::MaybeUninit,
        os::fd::{AsRawFd, OwnedFd},
    },
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
        let result: io::Result<OwnedFd> = unsafe {
            syscall!(
                SecComp,
                c::SET_MODE_FILTER,
                c::Filter::NEW_LISTENER.bits(),
                program.as_ptr()
            )
        };

        result.map(|fd| Listener { fd })
    }

    /// Blocks until the installed BPF program triggers [`Action::UserNotify`][bpf::Action::UserNotify].
    pub fn recv(&mut self) -> io::Result<Notification<'_>> {
        let mut notification = MaybeUninit::uninit();

        unsafe {
            c::recv(self.fd.as_raw_fd(), notification.as_mut_ptr())?;

            Ok(Notification {
                listener: self,
                notification: MaybeUninit::assume_init(notification),
            })
        }
    }
}

impl<'listener> Notification<'listener> {
    pub fn send_error(&mut self, error: i32) -> io::Result<()> {
        let response = c::seccomp_notif_resp {
            id: self.notification.id,
            error,
            ..Default::default()
        };

        self.send_response(response)
    }

    fn send_response(&mut self, response: c::seccomp_notif_resp) -> io::Result<()> {
        unsafe {
            c::send(
                self.listener.fd.as_raw_fd(),
                (&response as *const c::seccomp_notif_resp).cast_mut(),
            )?;
        }

        Ok(())
    }
}
