use {
    crate::{index, Index},
    mocha_fs::{Utf8Path, Utf8PathBuf},
    std::{io, time::Instant},
};

/// The main context/entry point for doing things with Milk.
pub struct Milk {
    prefix_dir: Utf8PathBuf,
    images_dir: Utf8PathBuf,
    repositories_dir: Utf8PathBuf,
    system_dir: Utf8PathBuf,
}

impl Milk {
    /// Create a new instance using the prefix directory, `prefix_dir`.
    pub fn new<P>(prefix_dir: P) -> Self
    where
        P: Into<Utf8PathBuf>,
    {
        let prefix_dir = prefix_dir.into();
        let images_dir = prefix_dir.join("images");
        let repositories_dir = prefix_dir.join("repositories");
        let system_dir = prefix_dir.join("system");

        Self {
            prefix_dir,
            images_dir,
            repositories_dir,
            system_dir,
        }
    }

    /// Returns the prefix directory.
    #[inline]
    pub fn prefix_dir(&self) -> &Utf8Path {
        &self.prefix_dir
    }

    /// Returns the directory where package images are stored.
    #[inline]
    pub fn images_dir(&self) -> &Utf8Path {
        &self.images_dir
    }

    /// Returns the directory where package repositories are stored locally.
    #[inline]
    pub fn repositories_dir(&self) -> &Utf8Path {
        &self.repositories_dir
    }

    /// Returns the directory where package images are mounted to.
    #[inline]
    pub fn system_dir(&self) -> &Utf8Path {
        &self.system_dir
    }

    /// "Open" the package index.
    ///
    /// Recurses installed package images, and package repositories.
    pub fn open_index(&self) -> io::Result<Index> {
        let start_time = Instant::now();
        let installed = mocha_fs::read_files_at(self.images_dir(), 1)
            .flatten()
            .flat_map(|entry| {
                let (stem, extension) = parts_of(&entry)?;
                let (ident, hash) = stem.rsplit_once('-')?;
                let ident = ident.parse().ok()?;

                Some(ident)
            })
            .collect();

        let map = mocha_fs::read_files_at(self.repositories_dir(), 2)
            .flatten()
            .flat_map(|entry| index::entry(self, &installed, entry))
            .collect();

        let index = Index { map };
        let elapsed = start_time.elapsed();
        let count = index.len();
        let pluralized = if count == 1 { "entry" } else { "entries" };

        println!("Package index loaded in {elapsed:.2?} ({count} {pluralized}).");

        Ok(index)
    }
}

/// Return both the file stem and extension of a `FileEntry`.
#[inline]
pub(crate) fn parts_of(entry: &mocha_fs::FileEntry) -> Option<(&str, &str)> {
    entry.file_name().rsplit_once('.')
}
