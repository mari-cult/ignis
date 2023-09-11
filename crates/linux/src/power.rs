//! Power-related functionality.
//!
//! # References
//!
//! - [`arch/arm64/kernel/process.c#L126`](https://github.com/torvalds/linux/blob/master/arch/arm64/kernel/process.c#L126)
//! - [`arch/x86/kernel/reboot.c#L763`](https://github.com/torvalds/linux/blob/master/arch/x86/kernel/reboot.c#L763)
//! - [`include/uapi/linux/reboot.h`](https://github.com/torvalds/linux/blob/master/include/uapi/linux/reboot.h)

use {crate::syscall::syscall, core::ffi::CStr, std::io};

const MAGIC1: u32 = 0xfee1dead;
const MAGIC2: u32 = 672274793;

const CAD_ON: u32 = 0x89ABCDEF;
const CAD_OFF: u32 = 0x00000000;
const POWER_OFF: u32 = 0x4321FEDC;
#[allow(dead_code)]
const RESTART2: u32 = 0xA1B2C3D4;
const RESTART: u32 = 0x01234567;

/// How to restart.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Restart {
    /// Normal restart.
    #[default]
    Normal,

    /// Download mode.
    ///
    /// Commonly found in smartphones.
    Download,

    /// UEFI firmware.
    ///
    /// Commonly found in personal computers.
    Firmware,

    /// Recocery.
    ///
    /// Commonly found in smartphones.
    Recovery,
}

impl Restart {
    #[allow(dead_code)]
    fn is_firmware(&self) -> bool {
        matches!(self, Self::Firmware)
    }

    #[allow(dead_code)]
    fn target(&self) -> Option<&'static CStr> {
        let string = match self {
            Self::Download => c"download",
            Self::Recovery => c"recovery",
            _ => return None,
        };

        Some(string)
    }
}

/// Issue a `reboot(2)` call that returns on success.
///
/// For `CAD_ON`, and `CAD_OFF`,
unsafe fn reboot(command: u32) -> io::Result<()> {
    syscall!(Reboot, MAGIC1, MAGIC2, command)
}

/// Issue a `reboot(2)` call that does not return on success.
///
/// For `POWER_OFF`, and `RESTART`.
unsafe fn reboot_noreturn(command: u32) -> io::Error {
    syscall!(Reboot, MAGIC1, MAGIC2, command)
}

/// Issue a `RESTART` reboot(2)` call that does not return on success.
fn do_restart() -> io::Error {
    unsafe { reboot_noreturn(RESTART) }
}

/// Power off the current device.
///
/// On success this function will not return,
/// and otherwise it will return an error
/// indicating why the power off failed.
pub fn power_off() -> io::Error {
    unsafe { reboot_noreturn(POWER_OFF) }
}

/// Restart the current device.
///
/// On success this function will not return,
/// and otherwise it will return an error
/// indicating why the restart failed.
pub fn restart(restart: Restart) -> io::Error {
    // Set EFI OS indications as the kernel ignores it.
    //
    // See https://github.com/torvalds/linux/blob/master/arch/x86/kernel/reboot.c#L763
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        use crate::efi;

        if restart.is_firmware() {
            // Only check if a firmware boot is requested.
            if efi::os_indications_supported() {
                if let Err(error) = efi::set_boot_to_firmware(true) {
                    return error;
                }
            }
        }

        do_restart()
    }

    // Let the kernel decide.
    //
    // See https://github.com/torvalds/linux/blob/master/arch/arm64/kernel/process.c#L126
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        if let Some(target) = restart.target() {
            // Issue a `RESTART2` reboot(2)` call that does not return on success.
            unsafe { syscall!(Reboot, MAGIC1, MAGIC2, RESTART2, target.as_ptr()) }
        } else {
            do_restart()
        }
    }
}

/// Enable or disable the [`Control-Alt-Delete`](https://en.wikipedia.org/wiki/Control-Alt-Delete) keyboard command.
pub fn set_control_alt_delete(enabled: bool) -> io::Result<()> {
    let command = if enabled { CAD_ON } else { CAD_OFF };

    unsafe { reboot(command) }
}
