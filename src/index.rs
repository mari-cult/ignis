use {
    crate::{milk, Milk},
    itertools::Itertools,
    mocha_fs::{Utf8Path, Utf8PathBuf},
    mocha_ident::{Artifact, CargoFeature, PackageIdent, RepositoryIdent, Source},
    serde::{Deserialize, Serialize},
    std::{
        collections::{BTreeSet, HashMap, HashSet},
        error, fmt, io,
    },
};

/// Package index.
pub struct Index {
    // Map of all installed packages.
    pub(crate) map: HashMap<PackageIdent, Entry>,
}

/// A package.
pub struct Package {
    ident: PackageIdent,
    entry: Entry,
}

#[derive(Debug)]
pub(crate) struct Entry {
    /// Whether this package is installed.
    installed: bool,
    /// Image path.
    image_path: Utf8PathBuf,
    /// System path.
    system_dir: Utf8PathBuf,
    /// Origin repository.
    repository: RepositoryIdent,
    /// Serialized package information.
    serialized: Serialized,
}

/// Serialized package information.
#[derive(Debug, Deserialize, Serialize)]
pub struct Serialized {
    sources: BTreeSet<Source>,
    parts: Vec<Part>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
// TODO: Use faster collections than BTreeSet's, and stricter validation.
/// A part to assemble a package.
pub enum Part {
    Rust {
        #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
        features: BTreeSet<CargoFeature>,
        #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
        depends: BTreeSet<PackageIdent>,
        artifacts: BTreeSet<String>,
    },
    CCpp {
        #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
        depends: BTreeSet<PackageIdent>,
        artifacts: BTreeSet<Artifact>,
    },
    Copy {
        #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
        depends: BTreeSet<PackageIdent>,
        artifacts: BTreeSet<Artifact>,
    },
    Zig {
        #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
        depends: BTreeSet<PackageIdent>,
        artifacts: BTreeSet<Artifact>,
    },
}

impl Index {
    /// Returns the count of packages within the index.
    #[inline]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// If there are no packages.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Resolve packages from the provided iterator.
    ///
    /// This consumes the index to free memory consumed by unused packages.
    pub fn resolve<I>(mut self, packages: I) -> Result<Vec<Package>, Vec<PackageIdent>>
    where
        I: IntoIterator<Item = PackageIdent>,
    {
        let (packages, unknown_packages): (Vec<_>, Vec<_>) = packages
            .into_iter()
            .map(|ident| self.map.remove_entry(&ident).ok_or(ident))
            .partition_result();

        if unknown_packages.is_empty() {
            // Convert `Vec<(PackageIdent, Entry)>` to `Vec<Package>`.
            // AFAIK, LLVM should optimize this out completely, as the representation is the same.
            let packages = packages
                .into_iter()
                .map(|(ident, entry)| Package { ident, entry })
                .collect::<Vec<_>>();

            Ok(packages)
        } else {
            Err(unknown_packages)
        }
    }
}

impl Package {
    /// This package's identifier.
    #[inline]
    pub fn ident(&self) -> PackageIdent {
        self.ident
    }

    /// Is this package installed.
    #[inline]
    pub fn is_installed(&self) -> bool {
        self.entry.installed
    }

    /// The image path.
    #[inline]
    pub fn image_path(&self) -> &Utf8Path {
        &self.entry.image_path
    }

    /// The system path.
    #[inline]
    pub fn system_dir(&self) -> &Utf8Path {
        &self.entry.system_dir
    }

    pub fn sources(&self) -> impl Iterator<Item = &Source> {
        self.entry.serialized.sources.iter()
    }

    pub fn parts(&self) -> &[Part] {
        &self.entry.serialized.parts
    }

    /// Attempt to uninstall this package.
    pub fn uninstall(&self) -> io::Result<()> {
        if !self.is_installed() {
            return Ok(());
        }

        mocha_fs::remove_mount(self.system_dir())?;
        mocha_fs::remove_dir(self.system_dir())?;
        mocha_fs::remove_file(self.image_path())?;

        Ok(())
    }
}

impl fmt::Debug for Index {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.map.keys(), fmt)
    }
}

/// Load a package entry.
pub(crate) fn entry(
    milk: &Milk,
    installed: &HashSet<PackageIdent>,
    entry: mocha_fs::FileEntry,
) -> io::Result<(PackageIdent, Entry)> {
    let (ident, extension) =
        milk::parts_of(&entry).ok_or_else(|| invalid_data("missing file stem and extension"))?;

    if extension != "toml" {
        return Err(invalid_data("not a spec"));
    }

    let ident = ident.parse()?;
    let repository = entry.parent().file_name().unwrap().parse()?;
    let specification = std::fs::read_to_string(entry.path())?;
    let serialized = toml::from_str(&specification).map_err(|error| {
        eprintln!("{ident}: {error}");

        io::Error::new(io::ErrorKind::InvalidData, error)
    })?;

    let image_path = milk.images_dir().join(ident).with_extension("mocha");
    let system_dir = milk.system_dir().join(ident);
    let entry = Entry {
        installed: installed.contains(&ident),
        image_path,
        system_dir,
        repository,
        serialized,
    };

    Ok((ident, entry))
}

#[inline]
fn invalid_data<E>(error: E) -> io::Error
where
    E: Into<Box<dyn error::Error + Send + Sync>>,
{
    io::Error::new(io::ErrorKind::InvalidData, error)
}
