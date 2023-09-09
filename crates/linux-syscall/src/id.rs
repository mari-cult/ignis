use linux_raw_sys::general::*;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u32)]
pub enum Id {
    Read = __NR_read,
    Reboot = __NR_reboot,
    SecComp = __NR_seccomp,
    Write = __NR_write,
}
