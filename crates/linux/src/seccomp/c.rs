pub const SET_MODE_FILTER: u32 = 1;
pub const USER_NOTIF_FLAG_CONTINUE: u32 = 1;

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct Filter: u32 {
        const TSYNC = 1 << 0;
        const LOG = 1 << 1;
        const SPEC_ALLOW = 1 << 2;
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

nix::ioctl_readwrite!(recv, b'!', 0, seccomp_notif);
nix::ioctl_readwrite!(send, b'!', 1, seccomp_notif_resp);
