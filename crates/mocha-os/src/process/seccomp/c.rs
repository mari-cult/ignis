//! https://github.com/torvalds/linux/blob/master/include/uapi/linux/seccomp.h

use std::ffi;

pub const SECCOMP_IOC_MAGIC: u8 = b'!';

pub const SECCOMP_SET_MODE_FILTER: u32 = 1;
pub const SECCOMP_USER_NOTIF_FLAG_CONTINUE: u32 = 1;

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct seccomp_data {
    pub nr: ffi::c_int,
    pub arch: u32,
    pub instruction_pointer: u64,
    pub args: [u64; 6],
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct seccomp_notif_sizes {
    pub seccomp_notif: u16,
    pub seccomp_notif_resp: u16,
    pub seccomp_data: u16,
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct seccomp_notif {
    pub id: u64,
    pub pid: u32,
    pub flags: u32,
    pub data: seccomp_data,
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct seccomp_notif_resp {
    pub id: u64,
    pub val: i64,
    pub error: i32,
    pub flags: u32,
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct seccomp_notif_addfd {
    pub id: u64,
    pub flags: u32,
    pub srcfd: u32,
    pub newfd: u32,
    pub newfd_flags: u32,
}
