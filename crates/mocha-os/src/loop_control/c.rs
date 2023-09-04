//! https://github.com/torvalds/linux/blob/master/include/uapi/linux/loop.h

use std::os::unix::io::RawFd;

pub const LOOP_CTL_ADD: u32 = 0x4C80;
pub const LOOP_CTL_REMOVE: u32 = 0x4C81;
pub const LOOP_CTL_GET_FREE: u32 = 0x4C82;

pub const LOOP_CONFIGURE: u32 = 0x4C0A;

pub const LO_NAME_SIZE: usize = 64;
pub const LO_KEY_SIZE: usize = 32;

bitflags::bitflags! {
    /// Flags used in `LoopOptions`.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct LoopFlags: u32 {
        const READ_ONLY = 1;
        const AUTO_CLEAR = 4;
        const PART_SCAN = 8;
        const DIRECT_IO = 16;
    }
}

/// A field of `loop_config`.
#[derive(Debug)]
#[repr(C)]
pub struct loop_info64 {
    pub lo_device: u64,
    pub lo_inode: u64,
    pub lo_rdevice: u64,
    pub lo_offset: u64,
    pub lo_sizelimit: u64,
    pub lo_number: u32,
    pub lo_encrypt_type: u32,
    pub lo_encrypt_key_size: u32,
    pub lo_flags: u32,
    pub lo_file_name: [u8; LO_NAME_SIZE],
    pub lo_crypt_name: [u8; LO_NAME_SIZE],
    pub lo_encrypt_key: [u8; LO_KEY_SIZE],
    pub lo_init: [u64; 2],
}

/// Used in the `LOOP_CONFIGURE` `ioctl`.
#[derive(Debug, Default)]
#[repr(C)]
pub struct loop_config {
    pub fd: RawFd,
    pub block_size: u32,
    pub info: loop_info64,
    pub reserved: [u64; 8],
}

// Unfortunate manual implementation as `Default` isn't implemented for `[T; N]` where `N > 32`.
impl Default for loop_info64 {
    #[inline]
    fn default() -> Self {
        Self {
            lo_device: 0,
            lo_inode: 0,
            lo_rdevice: 0,
            lo_offset: 0,
            lo_sizelimit: 0,
            lo_number: 0,
            lo_encrypt_type: 0,
            lo_encrypt_key_size: 0,
            lo_flags: 0,
            lo_file_name: [0; LO_NAME_SIZE],
            lo_crypt_name: [0; LO_NAME_SIZE],
            lo_encrypt_key: [0; LO_KEY_SIZE],
            lo_init: [0; 2],
        }
    }
}
