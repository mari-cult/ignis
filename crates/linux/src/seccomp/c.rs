use {
    super::bpf,
    crate::syscall::syscall,
    std::{
        io,
        mem::MaybeUninit,
        os::unix::io::{AsFd, AsRawFd, OwnedFd},
    },
};

pub const SECCOMP_IOC_MAGIC: u8 = b'!';
pub const SECCOMP_SET_MODE_FILTER: u32 = 1;
pub const SECCOMP_USER_NOTIF_FLAG_CONTINUE: u32 = 1;

bitflags::bitflags! {
    /// SecComp filter flags.
    #[derive(Clone, Copy, Debug)]
    pub struct Filter: u32 {
        const TSYNC = 1 << 0;
        const LOG = 1 << 1;
        const SPEC_ALLOW = 1 << 2;
        /// Create a new listener.
        const NEW_LISTENER = 1 << 3;
        const TSYNC_ESRCH = 1 << 4;
        const WAIT_KILLABLE_RECV = 1 << 5;
    }
}

#[derive(Default)]
#[repr(C)]
pub struct seccomp_data {
    pub nr: u32,
    pub arch: u32,
    pub ip: u64,
    pub args: [u64; 6],
}

#[derive(Default)]
#[repr(C)]
pub struct seccomp_notif {
    pub id: u64,
    pub pid: u32,
    pub flags: u32,
    pub data: seccomp_data,
}

#[derive(Default)]
#[repr(C)]
pub struct seccomp_notif_resp {
    pub id: u64,
    pub val: i64,
    pub error: i32,
    pub flags: u32,
}

pub fn seccomp_new_listener(program: &bpf::Program<'_>) -> io::Result<OwnedFd> {
    unsafe {
        syscall!(
            SecComp,
            SECCOMP_SET_MODE_FILTER,
            Filter::NEW_LISTENER.bits(),
            program.as_ptr()
        )
    }
}

pub fn seccomp_ioctl_notif_recv<Fd: AsFd>(listener: Fd) -> io::Result<seccomp_notif> {
    // FIXME: Engineer own solution or use rustix.
    nix::ioctl_readwrite!(ioctl, SECCOMP_IOC_MAGIC, 0, seccomp_notif);

    let fd = listener.as_fd().as_raw_fd();
    let mut notif = MaybeUninit::zeroed();

    // SAFETY: seccomp_notif must be zeroed.
    unsafe {
        ioctl(fd, notif.as_mut_ptr())?;

        Ok(MaybeUninit::assume_init(notif))
    }
}

pub fn seccomp_ioctl_notif_send<Fd: AsFd>(
    listener: Fd,
    mut response: seccomp_notif_resp,
) -> io::Result<()> {
    // FIXME: Engineer own solution or use rustix.
    nix::ioctl_readwrite!(ioctl, SECCOMP_IOC_MAGIC, 1, seccomp_notif_resp);

    let fd = listener.as_fd().as_raw_fd();

    unsafe {
        ioctl(fd, &mut response as *mut seccomp_notif_resp)?;
    }

    Ok(())
}
