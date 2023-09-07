//! Device-specific functionality.

use {crate::sys, std::io};

/// Restart the current device.
///
/// On success this function will not return,
/// and otherwise it will return an error
/// indicating why the restart failed.
#[inline]
#[must_use = "the error should be handled"]
pub fn restart() -> io::Error {
    sys::power::restart()
}

/// Power off the current device.
///
/// On success this function will not return,
/// and otherwise it will return an error
/// indicating why the power off failed.
#[inline]
#[must_use = "the error should be handled"]
pub fn power_off() -> io::Error {
    sys::power::power_off()
}

/// Enable or disable booting to firmware.
#[inline]
pub fn set_boot_to_firmware(enabled: bool) -> io::Result<()> {
    sys::efi::set_boot_to_firmware(enabled)
}

/// Enable or disable the [`Control-Alt-Delete`](https://en.wikipedia.org/wiki/Control-Alt-Delete) keyboard command.
#[inline]
pub fn set_control_alt_delete(enabled: bool) -> io::Result<()> {
    sys::power::set_control_alt_delete(enabled)
}
