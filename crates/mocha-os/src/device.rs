use {
    nix::sys::reboot::{self, RebootMode},
    std::io,
};

/// If the `Ok` variant is infallible, then only return the `Err`.
fn reboot(mode: RebootMode) -> io::Error {
    match reboot::reboot(mode) {
        Ok(_infallible) => unreachable!(),
        Err(error) => error.into(),
    }
}

/// Restart the device.
pub fn restart() -> io::Error {
    reboot(RebootMode::RB_AUTOBOOT)
}

/// Power off the device.
pub fn power_off() -> io::Error {
    reboot(RebootMode::RB_POWER_OFF)
}
