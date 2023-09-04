use {
    rustix::{
        fs::{self, Mode},
        io::Errno,
        mount::{self, MountFlags},
    },
    std::{ffi::CStr, io},
};

macro_rules! cstr {
    ($cstr:literal) => {{
        const CSTR: &'static CStr = match CStr::from_bytes_with_nul(concat!($cstr, '\0').as_bytes())
        {
            Ok(cstr) => cstr,
            Err(_error) => panic!("invalid C string"),
        };

        CSTR
    }};
}

pub const DENY_SET_USER_ID: MountFlags = MountFlags::NOSUID;
pub const DONT_FOLLOW_LINKS: MountFlags = MountFlags::NOSYMFOLLOW;
pub const DONT_UPDATE_ACCESS_TIME: MountFlags = MountFlags::NOATIME.union(MountFlags::NODIRATIME);
pub const NOT_EXECUTABLE: MountFlags = MountFlags::NOEXEC;
pub const NO_DEVICE_FILES: MountFlags = MountFlags::NODEV;

pub const EMPTY: &CStr = cstr!("");
pub const PROC_DATA: &CStr = cstr!("hidepid=invisible");

pub mod kind {
    use std::ffi::CStr;

    pub const DEVPTS: &CStr = cstr!("devpts");
    pub const DEVTMPFS: &CStr = cstr!("devtmpfs");
    pub const PROC: &CStr = cstr!("proc");
    pub const RAMFS: &CStr = cstr!("ramfs");
    pub const SYSFS: &CStr = cstr!("sysfs");
}

pub mod path {
    use std::ffi::CStr;

    pub const DEV: &CStr = cstr!("/dev");
    pub const DEV_PTS: &CStr = cstr!("/dev/pts");
    pub const DEV_SHM: &CStr = cstr!("/dev/shm");
    pub const PROC: &CStr = cstr!("/proc");
    pub const SYS: &CStr = cstr!("/sys");
    pub const TMP: &CStr = cstr!("/tmp");
}

/// Mount or update the specified mount.
pub fn ensure_mount(
    source: &CStr,
    target: &CStr,
    file_system_type: &CStr,
    flags: MountFlags,
    data: &CStr,
) -> io::Result<()> {
    let Err(error) = mount::mount(source, target, file_system_type, flags, data) else {
        return Ok(());
    };

    // Returns EBUSY if `source` is already mounted.
    if error == Errno::BUSY {
        mount::mount_remount(target, flags, data)?;
    }

    Ok(())
}

/// Mount standard directories.
pub fn setup_standard() -> io::Result<()> {
    // devtmpfs /dev noatime,nodiratime
    ensure_mount(
        kind::DEVTMPFS,
        path::DEV,
        kind::DEVTMPFS,
        DENY_SET_USER_ID | DONT_UPDATE_ACCESS_TIME,
        EMPTY,
    )?;

    fs::mkdir(path::DEV_PTS, Mode::from_raw_mode(0o0755))?;
    fs::mkdir(path::DEV_SHM, Mode::from_raw_mode(0o1777))?;

    // devpts /dev/pts noatime,nodiratime
    ensure_mount(
        kind::DEVPTS,
        path::DEV_PTS,
        kind::DEVPTS,
        DENY_SET_USER_ID | DONT_UPDATE_ACCESS_TIME,
        EMPTY,
    )?;

    // proc /proc noatime,nodiratime hidepid=invisible
    ensure_mount(
        kind::PROC,
        path::PROC,
        kind::PROC,
        DENY_SET_USER_ID | DONT_UPDATE_ACCESS_TIME,
        PROC_DATA,
    )?;

    // sysfs /sys noatime,nodiratime
    ensure_mount(
        kind::SYSFS,
        path::SYS,
        kind::SYSFS,
        DENY_SET_USER_ID | DONT_UPDATE_ACCESS_TIME | NOT_EXECUTABLE,
        EMPTY,
    )?;

    // ramfs /tmp noatime,nodiratime
    ensure_mount(
        kind::RAMFS,
        path::TMP,
        kind::RAMFS,
        DENY_SET_USER_ID | DONT_UPDATE_ACCESS_TIME,
        EMPTY,
    )?;

    fs::chmod(path::TMP, Mode::from_raw_mode(0o777))?;

    Ok(())
}
