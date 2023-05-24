use {
    crate::{path, Utf8Path},
    std::io,
};

type Filter = fn(&walkdir::DirEntry) -> bool;

/// An iterator of files at a particular depth within a directory.
pub struct ReadFilesAt {
    iter: walkdir::FilterEntry<walkdir::IntoIter, Filter>,
}

/// A file entry.
pub struct FileEntry {
    entry: walkdir::DirEntry,
}

impl ReadFilesAt {
    pub(crate) fn new<P: AsRef<Utf8Path>>(path: P, depth: u8) -> Self {
        let iter = walkdir::WalkDir::new(path.as_ref())
            .min_depth(depth as usize)
            .max_depth(depth as usize)
            .same_file_system(true)
            .into_iter()
            .filter_entry((|entry| entry.file_type().is_file()) as Filter);

        Self { iter }
    }
}

impl FileEntry {
    /// The full path that this entry represents.
    #[inline]
    pub fn path(&self) -> &Utf8Path {
        // SAFETY: UTF-8 validation is performed before constructing `FileEntry`.
        unsafe { path::from_utf8_unchecked(self.entry.path()) }
    }

    /// Return the file name of this entry.
    pub fn file_name(&self) -> &str {
        // SAFETY: All entries have file names.
        unsafe { self.path().file_name().unwrap_unchecked() }
    }

    /// The full parent path of this entry.
    #[inline]
    pub fn parent(&self) -> &Utf8Path {
        // SAFETY: A minimum of one parent is required to produce a `FileEntry`.
        unsafe { self.path().parent().unwrap_unchecked() }
    }
}

impl Iterator for ReadFilesAt {
    type Item = io::Result<FileEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.iter.next()?;
        let item = item.map_err(Into::into).and_then(|entry| {
            path::from_utf8(entry.path())?;

            Ok(FileEntry { entry })
        });

        Some(item)
    }
}
