use {
    camino::Utf8Path,
    petgraph::Graph,
    serde::{Deserialize, Serialize},
    std::{
        collections::{BTreeSet, HashMap},
        fmt, io,
    },
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Serialized {
    pub sources: BTreeSet<String>,
    pub parts: Vec<Part>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", untagged)]
pub enum Part {
    Rust {
        #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
        features: BTreeSet<String>,
        #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
        depends: BTreeSet<String>,
        artifacts: BTreeSet<String>,
    },
    CCpp {
        #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
        depends: BTreeSet<String>,
        artifacts: BTreeSet<String>,
    },
    Copy {
        #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
        depends: BTreeSet<String>,
        artifacts: BTreeSet<String>,
    },
    Zig {
        #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
        depends: BTreeSet<String>,
        artifacts: BTreeSet<String>,
    },
}

#[derive(Debug)]
pub struct Entry {
    pub installed: bool,
    pub repository: Box<str>,
    pub serialized: Serialized,
}

pub struct Index {
    pub graph: Graph<Box<str>, Box<str>>,
    pub index: HashMap<Box<str>, Entry>,
}

impl Index {
    pub fn open() -> io::Result<Self> {
        let mut graph = Graph::new();
        let mut index = HashMap::new();
        let mut mochas: HashMap<String, ()> = HashMap::new();

        for entry in read_dir("/mocha/images", 1) {
            let Some(path) = Utf8Path::from_path(entry.path()) else {
                continue;
            };

            let name = unsafe { path.file_name().unwrap_unchecked() };

            let Some((name, "mocha")) = name.rsplit_once('.') else {
                continue;
            };

            let Some((name, _hash)) = name.split_once('-') else {
                continue;
            };

            mochas.insert(name.into(), ());
        }

        for entry in read_dir("/mocha/repositories", 2) {
            let Some(path) = Utf8Path::from_path(entry.path()) else {
                continue;
            };

            let name = unsafe { path.file_name().unwrap_unchecked() };

            let Some((name, "spec")) = name.rsplit_once('.') else {
                continue;
            };

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
                    println!("{name}: {error:?}");

                    continue;
                }
            };

            graph.add_node(Box::from(name));

            index.insert(
                Box::from(name),
                Entry {
                    installed: mochas.contains_key(&name.to_string()),
                    repository: Box::from(repository),
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
