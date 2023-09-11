//! Linux-specific APIs.

#![deny(invalid_reference_casting)]
#![deny(missing_docs)]
#![deny(warnings)]
#![feature(c_str_literals)]
#![feature(unix_socket_ancillary_data)]

pub use linux_syscall as syscall;

pub mod efi;
pub mod fd;
pub mod power;
pub mod process;
pub mod seccomp;
