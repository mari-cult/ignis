use crate::macros::ids;

ids! {
    Exit = __NR_exit,
    ExitGroup = __NR_exit_group,
    GetGid = __NR_getgid,
    GetUid = __NR_getuid,
    Read = __NR_read,
    Reboot = __NR_reboot,
    SecComp = __NR_seccomp,
    Write = __NR_write,
}
