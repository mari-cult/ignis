//! Linux-specific APIs.

#![deny(invalid_reference_casting)]
#![deny(missing_docs)]
#![deny(warnings)]

pub use linux_syscall as syscall;

pub mod efi;
pub mod power;
pub mod process;
pub mod seccomp;

pub(crate) mod util;
