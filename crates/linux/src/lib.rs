//! Linux-specific APIs.

pub use linux_syscall as syscall;

pub mod efi;
pub mod power;
pub mod process;
pub mod seccomp;

pub(crate) mod util;
