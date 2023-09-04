use {
    rustix::{
        process::{self, Pid, WaitOptions, WaitStatus},
        thread::{self, RawPid},
    },
    std::io,
};

pub mod seccomp;

/// Create a new process session, and become the process group leader.
///
/// Returns `PermissionDenied` if this process is already the leader.
pub fn create_new_session() -> io::Result<u32> {
    let id = process::setsid()?;

    Ok(id.as_raw_nonzero().get() as u32)
}

/// Promise no new permissions will be granted/
///
/// Renders setting user/group ids useless.
pub fn deny_new_permissions() -> io::Result<()> {
    thread::set_no_new_privs(true)?;

    Ok(())
}

/// Wait on all child processes.
pub fn wait_all<F: FnMut(WaitStatus)>(mut f: F) -> io::Result<()> {
    while let Some(status) = process::waitpid(Pid::from_raw(RawPid::MAX), WaitOptions::NOHANG)? {
        (f)(status);
    }

    Ok(())
}
