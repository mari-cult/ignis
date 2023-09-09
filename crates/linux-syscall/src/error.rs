//! # References
//!
//! - [GNU libc Error Codes](https://www.gnu.org/software/libc/manual/html_node/Error-Codes.html)

use core::fmt;

macro_rules! error {
    ($(
        $(#[$meta:meta])*
        $ident:ident = ($value:ident, $description:literal),
    )*) => {
        /// An Error.
        ///
        /// Heavily inspired by [`io::Error`](std::io::Error),
        #[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
        pub enum Error {
            $(
                $(#[$meta])*
                $ident,
            )*
        }

        impl Error {
            /// Creates a new instance of an `Error` from a particular OS error code.
            ///
            /// This conversion is lossy, unknown OS error codes become `Uncategorized`.
            pub fn from_raw_os_error(error: i32) -> Self {
                match error {
                    $(errno::$value => Self::$ident,)*
                    _ => Self::Uncategorized,
                }
            }

            /// Returns the OS error that this error represents.
            pub fn raw_os_error(&self) -> i32 {
                match self {
                    $(Self::$ident => errno::$value,)*
                }
            }

            /// Returns a description for this error.
            pub fn description(&self) -> &'static str {
                match self {
                    $(Self::$ident => $description,)*
                }
            }
        }
    };
}

mod errno {
    use linux_raw_sys::errno;

    pub const INVALID_INPUT: i32 = errno::EINVAL as i32;
    pub const NOT_FOUND: i32 = errno::ENOENT as i32;
    pub const PERMISSION_DENIED: i32 = errno::EACCES as i32;
    pub const RESOURCE_BUSY: i32 = errno::EBUSY as i32;
    pub const UNKNOWN: i32 = 0;
}

error! {
    /// A parameter is incorrect.
    InvalidInput = (INVALID_INPUT, "a parameter is incorrect"),

    /// An entity was not found, often a file.
    NotFound = (NOT_FOUND, "an entity was not found"),

    /// The operation lacked the necessary privileges to complete.
    PermissionDenied = (PERMISSION_DENIED, "permission denied"),

    /// A resource is busy.
    ResourceBusy = (RESOURCE_BUSY, "a resource is busy"),

    /// An uncategorized error.
    #[default]
    Uncategorized = (UNKNOWN, "uncategorized"),
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

// Conversion to nix's Errno.
#[cfg(feature = "nix")]
impl From<Error> for nix::errno::Errno {
    fn from(error: Error) -> Self {
        nix::errno::from_i32(error.raw_os_error())
    }
}

// Conversion from nix's Errno.
#[cfg(feature = "nix")]
impl From<nix::errno::Errno> for Error {
    fn from(error: nix::errno::Errno) -> Self {
        Self::from_raw_os_error(error as i32)
    }
}

// Conversion to rustix's Errno.
#[cfg(feature = "rustix")]
impl From<Error> for rustix::io::Errno {
    fn from(error: Error) -> Self {
        rustix::io::Errno::from_raw_os_error(error.raw_os_error())
    }
}

// Conversion from rustix's Errno.
#[cfg(feature = "rustix")]
impl From<rustix::io::Errno> for Error {
    fn from(error: rustix::io::Errno) -> Self {
        Self::from_raw_os_error(error.raw_os_error())
    }
}

#[allow(unused_macros)]
macro_rules! error_kind {
    ($($error:ident => $std:ident,)*) => {
        // Conversion to std's ErrorKind.
        impl From<Error> for std::io::ErrorKind {
            /// This conversion is lossy.
            fn from(error: Error) -> Self {
                use std::io::ErrorKind;

                match error {
                    $(Error::$error => ErrorKind::$std,)*
                    _ => ErrorKind::Other,
                }
            }
        }

        // Conversion from std's ErrorKind.
        impl From<std::io::ErrorKind> for Error {
            /// This conversion is lossy.
            fn from(error: std::io::ErrorKind) -> Self {
                use std::io::ErrorKind;

                match error {
                    $(ErrorKind::$std => Error::$error,)*
                    _ => Error::Uncategorized,
                }
            }
        }
    };
}

#[cfg(feature = "std")]
error_kind! {
    InvalidInput => InvalidInput,
    NotFound => NotFound,
    PermissionDenied => PermissionDenied,
}

// Conversion to std's Error.
#[cfg(feature = "std")]
impl From<Error> for std::io::Error {
    fn from(error: Error) -> Self {
        std::io::Error::from_raw_os_error(error.raw_os_error() as _)
    }
}

// Conversion from std's Error.
#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
    /// This conversion is lossy.
    fn from(error: std::io::Error) -> Self {
        error
            .raw_os_error()
            .map(Self::from_raw_os_error)
            .unwrap_or_else(|| error.kind().into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(self.description())
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::Error;

    #[cfg(feature = "nix")]
    #[test]
    fn from_nix() {
        use nix::errno;

        let error: Error = errno::Errno::EPERM.into();
        let expected = Error::PermissionDenied;

        assert_eq!(error, expected);
    }

    #[cfg(feature = "nix")]
    #[test]
    fn into_nix() {
        use nix::errno;

        let error: errno::Errno = Error::PermissionDenied.into();
        let expected = errno::Errno::EPERM;

        assert_eq!(error, expected);
    }

    #[cfg(feature = "rustix")]
    #[test]
    fn from_rustix() {
        use rustix::io;

        let error: Error = io::Errno::PERM.into();
        let expected = Error::PermissionDenied;

        assert_eq!(error, expected);
    }

    #[cfg(feature = "rustix")]
    #[test]
    fn into_rustix() {
        use rustix::io;

        let error: io::Errno = Error::PermissionDenied.into();
        let expected = io::Errno::PERM;

        assert_eq!(error, expected);
    }

    #[cfg(feature = "std")]
    #[test]
    fn from_std_error() {
        use std::io;

        let error: io::Error = io::ErrorKind::PermissionDenied.into();
        let error: Error = error.into();
        let expected = Error::PermissionDenied;

        assert_eq!(error, expected);
    }

    #[cfg(feature = "std")]
    #[test]
    fn into_std_error() {
        use std::io;

        let error: io::Error = Error::PermissionDenied.into();
        let expected: io::Error = io::ErrorKind::PermissionDenied.into();

        assert_eq!(error.kind(), expected.kind());
    }
}
