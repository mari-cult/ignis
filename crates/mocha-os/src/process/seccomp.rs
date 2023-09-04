use {
    nix::{errno::Errno, libc},
    rustix::ioctl,
    std::{
        io,
        mem::MaybeUninit,
        os::unix::io::{AsFd, FromRawFd, OwnedFd, RawFd},
    },
};

pub mod bpf;
pub mod c;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Interrupt {
    Allow = 0x7fff0000,
    Errno = 0x00050000,
    KillProcess = 0x80000000,
    KillThread = 0x00000000,
    Log = 0x7ffc0000,
    Trace = 0x7ff00000,
    Trap = 0x00030000,
    UserNotif = 0x7fc00000,
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct FilterFlag: u32 {
        const TSYNC = 1 << 0;
        const LOG = 1 << 1;
        const SPEC_ALLOW = 1 << 2;
        const NEW_LISTENER = 1 << 3;
        const TSYNC_ESRCH = 1 << 4;
        const WAIT_KILLABLE_RECV = 1 << 5;
    }
}

/// Set SecComp BPF.
pub fn set_filter(program: &bpf::sock_fprog) -> io::Result<OwnedFd> {
    let raw_fd = unsafe {
        libc::syscall(
            libc::SYS_seccomp,
            c::SECCOMP_SET_MODE_FILTER,
            FilterFlag::NEW_LISTENER.bits(),
            program as *const bpf::sock_fprog,
        )
    };

    let fd = Errno::result(raw_fd)?;

    Ok(unsafe { OwnedFd::from_raw_fd(fd as RawFd) })
}

/// Blocks until the BPF program, associated with the provided listener
/// file descriptor, triggers a [`Interrupt::UserNotif`](Interrupt::UserNotif).
pub fn recv<Fd: AsFd>(fd: Fd) -> io::Result<c::seccomp_notif> {
    let mut request = MaybeUninit::uninit();

    unsafe {
        let ctl = ioctl::Setter::<
            ioctl::ReadWriteOpcode<{ c::SECCOMP_IOC_MAGIC }, 0, ()>,
            *mut c::seccomp_notif,
        >::new(request.as_mut_ptr());

        ioctl::ioctl(fd, ctl)?;

        Ok(MaybeUninit::assume_init(request))
    }
}

/// Responds to the last [`recv()`](recv).
pub fn send<Fd: AsFd>(fd: Fd, response: c::seccomp_notif_resp) -> io::Result<()> {
    unsafe {
        let ctl = ioctl::Setter::<
            ioctl::ReadWriteOpcode<{ c::SECCOMP_IOC_MAGIC }, 1, ()>,
            *const c::seccomp_notif_resp,
        >::new(&response);

        ioctl::ioctl(fd, ctl)?;

        Ok(())
    }
}
