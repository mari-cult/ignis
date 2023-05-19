use {
    camino::Utf8Path,
    mocha_ident::spec::{ArtifactIdent, FeatureIdent, PackageIdent, RepositoryIdent, SourceIdent},
    petgraph::Graph,
    serde::{Deserialize, Serialize},
    std::{
        collections::{BTreeSet, HashMap},
        fmt, io,
    },
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Serialized {
    pub sources: BTreeSet<SourceIdent>,
    pub parts: Vec<Part>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", untagged)]
pub enum Part {
    Rust {
        #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
        features: BTreeSet<FeatureIdent>,
        #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
        depends: BTreeSet<PackageIdent>,
        artifacts: BTreeSet<ArtifactIdent>,
    },
    CCpp {
        #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
        depends: BTreeSet<PackageIdent>,
        artifacts: BTreeSet<ArtifactIdent>,
    },
    Copy {
        #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
        depends: BTreeSet<PackageIdent>,
        artifacts: BTreeSet<ArtifactIdent>,
    },
    Zig {
        #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
        depends: BTreeSet<PackageIdent>,
        artifacts: BTreeSet<ArtifactIdent>,
    },
}

#[derive(Debug)]
pub struct Entry {
    pub installed: bool,
    pub repository: RepositoryIdent,
    pub serialized: Serialized,
}

pub struct Index {
    pub graph: Graph<PackageIdent, Box<str>>,
    pub index: HashMap<PackageIdent, Entry>,
}

impl Index {
    pub fn open() -> io::Result<Self> {
        let mut graph = Graph::new();
        let mut index = HashMap::new();
        let mut mochas: HashMap<PackageIdent, ()> = HashMap::new();

        for entry in read_dir("/mocha/images", 1) {
            let Some(path) = path_utf8(&entry) else {
                continue;
            };

            let Some((name, "mocha")) = file_stem_extension(path) else {
                continue;
            };

            let Some((name, _hash)) = name.split_once('-') else {
                continue;
            };

            let name = name.parse().unwrap();

            mochas.insert(name, ());
        }

        for entry in read_dir("/mocha/repositories", 2) {
            let Some(path) = path_utf8(&entry) else {
                continue;
            };

            let Some((name, "spec")) = file_stem_extension(path) else {
                continue;
            };

            let name = name.parse().unwrap();

            // SAFETY: minimum depth enforces a parent to exist.
            let repository = unsafe {
                path.parent()
                    .unwrap_unchecked()
                    .file_name()
                    .unwrap_unchecked()
            };

            let Ok(content) = std::fs::read_to_string(path) else {
                continue;
            };

            let serialized = match serde_yaml::from_str(&content) {
                Ok(serialized) => serialized,
                Err(error) => {
                    eprintln!("{name}: {error:?}");

                    continue;
                }
            };

            graph.add_node(name);

            index.insert(
                name,
                Entry {
                    installed: mochas.contains_key(&name),
                    repository: repository.parse().unwrap(),
                    serialized,
                },
            );
        }

        Ok(Self { graph, index })
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.index.len()
    }
}

impl fmt::Debug for Index {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.index, fmt)
    }
}

fn read_dir(dir: &str, depth: usize) -> impl Iterator<Item = walkdir::DirEntry> {
    walkdir::WalkDir::new(dir)
        .max_depth(depth)
        .min_depth(depth)
        .same_file_system(true)
        .into_iter()
        .flatten()
        .filter(|entry| entry.file_type().is_file())
}

fn path_utf8(entry: &walkdir::DirEntry) -> Option<&Utf8Path> {
    Utf8Path::from_path(entry.path())
}

fn file_stem_extension(path: &Utf8Path) -> Option<(&str, &str)> {
    let file_name = path.file_name()?;

    file_name.rsplit_once('.')
}
