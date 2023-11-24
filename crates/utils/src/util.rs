use {
    clap_complete::Shell,
    std::{
        ffi::{OsStr, OsString},
        fs::File,
        io::{self, Write},
        mem::ManuallyDrop,
        os::fd::{AsRawFd, FromRawFd},
        path::PathBuf,
    },
};

/// 8 KiB is a good, commonly used size to write efficiently.
const DEFAULT_BUF_SIZE: usize = 8192;

/// Join `text` into a line.
pub fn join_line(text: Vec<OsString>) -> OsString {
    let mut line = text.join(OsStr::new(" "));

    line.push("\n");
    line
}

/// Parse `KEY=value` from an [`OsString`].
pub fn key_value(string: OsString) -> Result<(OsString, OsString), &'static str> {
    match os_str_split_once(&string, b'=') {
        Some((key, value)) => Ok((OsString::from(key), OsString::from(value))),
        None => Err("expected KEY=value"),
    }
}

/// Parse a path to a shell.
pub fn shell(string: OsString) -> Result<Shell, &'static str> {
    let shell = PathBuf::from(string);

    let Some(shell) = shell.file_name() else {
        return Err("expected path or name of a shell");
    };

    shell
        .to_str()
        .and_then(|shell| shell.parse().ok())
        .ok_or("unsupported shell")
}

/// Split [`OsStr`] by `byte`.
pub fn os_str_split_once(os_str: &OsStr, byte: u8) -> Option<(&OsStr, &OsStr)> {
    let bytes = os_str.as_encoded_bytes();
    let position = memchr::memchr(byte, bytes)?;

    unsafe {
        Some((
            OsStr::from_encoded_bytes_unchecked(bytes.get_unchecked(..position)),
            OsStr::from_encoded_bytes_unchecked(bytes.get_unchecked((position + 1)..)),
        ))
    }
}

/// Repeatedly write `buf` into [`stdout`](io::stdout)
/// until the program is terminated, or an error occurs.
///
/// The implementation of this function attempts to be as performant as possible.
pub fn stdout_write_repeated(buf: &[u8]) -> io::Result<()> {
    // Copy `buf` until there is at least, an 8 KiB buffer.
    let buf = buf.repeat(DEFAULT_BUF_SIZE.checked_div(buf.len()).unwrap());

    // Lock stdout.
    // Ensure nothing can write to stdout during the below code.
    let lock = io::stdout().lock();

    // Obtain a reference to stdout that isn't buffered.
    // Internal implementation of stdout is buffered by a `LineWriter`.
    // SAFETY: `ManuallyDrop` ensures the stdout handle isnt closed on drop.
    let mut stdout = unsafe { ManuallyDrop::new(File::from_raw_fd(lock.as_raw_fd())) };

    // Write the at least 8 KiB buffer to stdout as fast as possible.
    loop {
        stdout.write_all(&buf)?;
    }
}

pub trait OsStrExt {
    /// Splits the string on the first occurrence of the specified delimiter and
    /// returns prefix before delimiter and suffix after delimiter.
    fn split_once<'a, P: AsRef<OsStr>>(&'a self, delimiter: P) -> Option<(&'a OsStr, &'a OsStr)>;
}

impl OsStrExt for OsStr {
    fn split_once<'a, P: AsRef<OsStr>>(&'a self, delimiter: P) -> Option<(&'a OsStr, &'a OsStr)> {
        let haystack = self.as_encoded_bytes();
        let needle = delimiter.as_ref().as_encoded_bytes();
        let position = memchr::memmem::find(haystack, needle)?;

        unsafe {
            Some((
                OsStr::from_encoded_bytes_unchecked(haystack.get_unchecked(..position)),
                OsStr::from_encoded_bytes_unchecked(
                    haystack.get_unchecked((position + needle.len())..),
                ),
            ))
        }
    }
}
