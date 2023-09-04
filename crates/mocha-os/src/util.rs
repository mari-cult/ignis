use {nix::errno::Errno as NixErrno, rustix::io::Errno as RustixErrno};

pub fn nix_to_rustix(errno: NixErrno) -> RustixErrno {
    RustixErrno::from_raw_os_error(errno as i32)
}
