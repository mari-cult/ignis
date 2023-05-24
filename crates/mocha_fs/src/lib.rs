//! Wrappers for various file system-related methods.
//!
//! UTF-8 paths are enforced everywhere.

use {
    rustix::fs::{self as rustix_fs, UnmountFlags},
    std::{
        fs::{self as std_fs, File},
        io::{self, BufReader, BufWriter},
        os::unix::fs as unix_fs,
    },
};

pub use {
    crate::{
        permissions::{Permission, Permissions},
        read_dir::{FileEntry, ReadFilesAt},
    },
    camino::{Utf8Path, Utf8PathBuf},
};

pub(crate) mod path;

mod permissions;
mod read_dir;

const REMOVE_MOUNT_FLAGS: UnmountFlags = UnmountFlags::NOFOLLOW;

/// Convenience method to open a file at `path` for buffered reading.
#[inline]
pub fn open_buffered<P: AsRef<Utf8Path>>(path: P) -> io::Result<BufReader<File>> {
    File::open(path.as_ref()).map(BufReader::new)
}

/// Convenience method to create a file at `path` for buffered writing.
#[inline]
pub fn create_buffered<P: AsRef<Utf8Path>>(path: P) -> io::Result<BufWriter<File>> {
    File::create(path.as_ref()).map(BufWriter::new)
}

/// Convenience method to create a new file at `path` for buffered writing.
#[inline]
pub fn create_new_buffered<P: AsRef<Utf8Path>>(path: P) -> io::Result<BufWriter<File>> {
    File::options()
        .create_new(true)
        .write(true)
        .open(path.as_ref())
        .map(BufWriter::new)
}

/// Read files in directory `path`, at depth `depth`.
///
/// Only recurses items on the same file system as `path`.
///
/// # Panics
///
/// `depth` must not be `0`.
#[inline]
pub fn read_files_at<P: AsRef<Utf8Path>>(path: P, depth: u8) -> ReadFilesAt {
    assert_ne!(depth, 0, "depth cannot be zero");

    ReadFilesAt::new(path, depth)
}

/// Remove a directory.
#[inline]
pub fn remove_dir<P: AsRef<Utf8Path>>(path: P) -> io::Result<()> {
    std_fs::remove_dir(path.as_ref())?;

    Ok(())
}

/// Remove a directory, and all of it's contents.
///
/// Does not follow symbolic links, removes the link instead.
#[inline]
pub fn remove_dir_all<P: AsRef<Utf8Path>>(path: P) -> io::Result<()> {
    std_fs::remove_dir_all(path.as_ref())?;

    Ok(())
}

/// Remove a file.
#[inline]
pub fn remove_file<P: AsRef<Utf8Path>>(path: P) -> io::Result<()> {
    std_fs::remove_file(path.as_ref())?;

    Ok(())
}

/// Removes a mount point.
///
/// Does not follow symbolic links.
#[inline]
pub fn remove_mount<P: AsRef<Utf8Path>>(path: P) -> io::Result<()> {
    rustix_fs::unmount(path.as_ref().as_std_path(), REMOVE_MOUNT_FLAGS)?;

    Ok(())
}

/// Create a symbolic link.
#[inline]
pub fn symbolic_link<P: AsRef<Utf8Path>, L: AsRef<Utf8Path>>(path: P, link: L) -> io::Result<()> {
    unix_fs::symlink(path.as_ref(), link.as_ref())?;

    Ok(())
}
