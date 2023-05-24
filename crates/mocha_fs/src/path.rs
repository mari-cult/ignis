use {
    crate::Utf8Path,
    std::{
        io::{self, Error, ErrorKind},
        path::Path,
    },
};

/// Converts a `Path` to `Utf8Path`.
///
/// Returns `ErrorKind::InvalidData` if it's not UTF-8.
pub fn from_utf8(path: &Path) -> io::Result<&Utf8Path> {
    Utf8Path::from_path(path).ok_or_else(|| Error::new(ErrorKind::InvalidData, "invalid utf-8"))
}

/// Converts a `Path` to `Utf8Path`, without checking.
///
/// # Safety
///
/// Caller must ensure `path` is valid UTF-8.
#[inline]
pub unsafe fn from_utf8_unchecked(path: &Path) -> &Utf8Path {
    #[cfg(debug_assertions)]
    {
        assert!(from_utf8(path).is_ok(), "path is invalid utf-8");
    }

    // SAFETY: `Utf8Path` is `repr(transparent)`.
    &*(path as *const Path as *const Utf8Path)
}
