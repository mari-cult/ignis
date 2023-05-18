use std::io::{Error, ErrorKind};

pub fn must_be_an_exe() -> Error {
    Error::new(ErrorKind::PermissionDenied, "cargo must be an executable")
}

pub fn missing_stdio() -> Error {
    Error::new(ErrorKind::NotFound, "missing stdio handle")
}

pub fn invalid_message() -> Error {
    Error::new(ErrorKind::InvalidData, "invalid message from cargo")
}

pub fn invalid_plan(error: serde_json::Error) -> Error {
    Error::new(ErrorKind::InvalidData, error)
}

pub fn already_exited() -> Error {
    Error::new(ErrorKind::NotFound, "already exited")
}
