#![allow(non_snake_case)]

use {memoffset::offset_of, nix::libc::seccomp_data};

pub use nix::libc::{
    sock_filter, sock_fprog, BPF_ABS as ABS, BPF_JEQ as JEQ, BPF_JMP as JMP, BPF_K as K,
    BPF_LD as LD, BPF_RET as RET, BPF_W as W,
};

pub const LOAD_SYSCALL_NR: sock_filter =
    STMT((LD + W + ABS) as u16, offset_of!(seccomp_data, nr) as u32);

#[inline]
pub const fn JUMP(code: u16, k: u32, jt: u8, jf: u8) -> sock_filter {
    sock_filter { code, jt, jf, k }
}

#[inline]
pub const fn STMT(code: u16, k: u32) -> sock_filter {
    JUMP(code, k, 0, 0)
}

#[inline]
pub const fn ACTION(action: u32) -> sock_filter {
    STMT((RET + K) as u16, action)
}

#[inline]
pub const fn SYSCALL(nr: u32) -> sock_filter {
    JUMP((JMP + JEQ + K) as u16, nr, 0, 1)
}
