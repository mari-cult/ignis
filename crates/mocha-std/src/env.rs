use std::env;

/// `true` if under an ssh shell, `false` otherwise.
pub fn is_ssh() -> bool {
    env::var_os("SSH_CONNECTION").is_some()
}
