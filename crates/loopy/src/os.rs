//! See `include/uapi/linux/loop.h`, in Linux's source for more details on ioctls and structures used henceforth.

use std::{fs::File, io, os::unix::io::AsRawFd};

const LO_NAME_SIZE: usize = 64;
const LO_KEY_SIZE: usize = 32;

/// A structure representing `/dev/loop-control`.
#[derive(Debug)]
pub struct LoopControl {
    device: File,
}

/// A structure representing `/dev/loop{index}`.
#[derive(Debug)]
pub struct Loop {
    device: File,
}

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

/// `/dev/loop{index}` options.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LoopOptions {
    pub offset: u64,
    pub flags: LoopFlags,
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
    pub fd: u32,
    pub block_size: u32,
    pub info: loop_info64,
    pub reserved: [u64; 8],
}

impl LoopControl {
    /// Open `/dev/loop-control`.
    pub fn open() -> io::Result<Self> {
        let device = File::options()
            .read(true)
            .write(true)
            .open("/dev/loop-control")?;

        Ok(LoopControl { device })
    }

    /// Find, or allocate a free `/dev/loop{index}` device.
    pub fn next(&mut self) -> io::Result<Loop> {
        nix::ioctl_none_bad!(loop_ctl_get_free, 0x4C82);

        let index = unsafe { loop_ctl_get_free(self.device.as_raw_fd())? };
        let device = File::options()
            .read(true)
            .write(true)
            .open(format!("/dev/loop{index}"))?;

        Ok(Loop { device })
    }
}

impl Loop {
    /// Set the backing file descriptor of this `/dev/loop{index}` device.
    pub fn set_file<F: AsRawFd>(&mut self, file: &F, options: LoopOptions) -> io::Result<()> {
        nix::ioctl_write_ptr_bad!(loop_configure, 0x4C0A, loop_config);

        let config = loop_config {
            fd: file.as_raw_fd() as u32,
            block_size: 4096,
            info: loop_info64 {
                lo_offset: options.offset,
                lo_flags: options.flags.bits(),
                ..Default::default()
            },
            ..Default::default()
        };

        unsafe {
            // TODO: `LOOP_CONFIGURE` support detection based on `https://android.googlesource.com/platform/system/apex/+/refs/heads/master/apexd/apexd_loop.cpp`.
            loop_configure(self.device.as_raw_fd(), &config)?;
        }

        Ok(())
    }
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
