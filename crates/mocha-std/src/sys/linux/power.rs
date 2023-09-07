//! Power-related functions.

use {
    nix::sys::reboot::{self, RebootMode},
    std::io,
};

/// Convert the `Result<Infallible, Errno>` to `io::Error`.
#[inline]
fn reboot(mode: RebootMode) -> io::Error {
    match reboot::reboot(mode) {
        Err(error) => error.into(),
        Ok(_infallible) => unreachable!(),
    }
}

/// Restart the current device.
///
/// On success this function will not return,
/// and otherwise it will return an error
/// indicating why the restart failed.
#[inline]
#[must_use = "the error should be handled"]
pub fn restart() -> io::Error {
    reboot(RebootMode::RB_AUTOBOOT)
}

/// Power off the current device.
///
/// On success this function will not return,
/// and otherwise it will return an error
/// indicating why the power off failed.
#[inline]
#[must_use = "the error should be handled"]
pub fn power_off() -> io::Error {
    reboot(RebootMode::RB_POWER_OFF)
}

/// Enable or disable the [`Control-Alt-Delete`](https://en.wikipedia.org/wiki/Control-Alt-Delete) keyboard command.
#[inline]
pub fn set_control_alt_delete(enabled: bool) -> io::Result<()> {
    reboot::set_cad_enabled(enabled)?;

    Ok(())
}
