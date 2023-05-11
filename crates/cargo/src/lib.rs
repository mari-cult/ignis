use {
    camino::{Utf8Path, Utf8PathBuf},
    cargo_metadata::Message,
    milk_target::Target,
    serde::Deserialize,
    std::{
        collections::BTreeSet,
        fs,
        io::{self, Cursor},
        os::unix::fs::PermissionsExt,
        process::Stdio,
    },
    tokio::{
        io::{AsyncBufReadExt, BufReader, Lines},
        process::{ChildStderr, ChildStdout, Command},
    },
};

/// A `cargo` invocation context.
pub struct Cargo {
    cargo_path: Utf8PathBuf,
}

/// A `cargo build` invocation builder.
pub struct Build {
    cargo_path: Utf8PathBuf,
    workspace_path: Utf8PathBuf,
    features: BTreeSet<String>,
    target: Target,
}

/// A `cargo build` child process.
pub struct Child {
    cargo_path: Utf8PathBuf,
    workspace_path: Utf8PathBuf,
    features: String,
    triple: String,
    child: tokio::process::Child,
    stdout: Lines<BufReader<ChildStdout>>,
    completed: usize,
    total: Option<usize>,
}

/// Child process status.
pub struct Status {
    pub completed: usize,
    pub total: usize,
}

#[derive(Debug, Deserialize)]
struct BuildPlan {
    invocations: Vec<serde_json::Value>,
}

impl Cargo {
    /// Create a new context.
    ///
    /// Ensures that `cargo_path` points to a valid executable.
    pub fn new<P>(cargo_path: P) -> io::Result<Self>
    where
        P: AsRef<Utf8Path>,
    {
        let cargo_path = cargo_path.as_ref().canonicalize_utf8()?;
        let metadata = fs::metadata(&cargo_path)?;

        // Is not a file.
        if !metadata.file_type().is_file() {
            return Err(cargo_must_be_an_exe());
        }

        // Is not executable by the user.
        if metadata.permissions().mode() & 0o100 != 0o100 {
            return Err(cargo_must_be_an_exe());
        }

        Ok(Self { cargo_path })
    }

    /// Create an invocation builder.
    pub fn build<P>(&self, workspace_path: P) -> Build
    where
        P: Into<Utf8PathBuf>,
    {
        Build {
            cargo_path: self.cargo_path.clone(),
            workspace_path: workspace_path.into(),
            features: BTreeSet::new(),
            target: Target::HOST,
        }
    }
}

impl Build {
    /// Append a cargo feature.
    pub fn feature<S>(mut self, feature: S) -> Self
    where
        S: Into<String>,
    {
        self.features.insert(feature.into());
        self
    }

    /// Append multiple cargo features.
    pub fn features<I, S>(mut self, features: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for feature in features {
            self = self.feature(feature);
        }

        self
    }

    /// Set the target to build for.
    pub fn target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }

    /// Start the build.
    pub fn spawn(self) -> io::Result<Child> {
        let Self {
            cargo_path,
            workspace_path,
            features,
            target,
        } = self;

        let features = features.into_iter().collect::<Vec<_>>().join(",");
        let triple = target.rust_triple().into();

        let mut child = Command::new(&cargo_path)
            .current_dir(&workspace_path)
            // Force a toolchain to ignore the `rust-toolchain` file in projects.
            .arg("+nightly")
            // Zig toolchain, my beloved.
            .arg("zigbuild")
            // Parse JSON messages.
            .arg("--message-format=json-render-diagnostics")
            .arg("--no-default-features")
            .arg("--release")
            .arg(format!("--features={features}"))
            .arg(format!("--target={triple}"))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let stdout = child.stdout.take().ok_or_else(missing_stdio)?;
        let stdout = BufReader::new(stdout).lines();

        Ok(Child {
            cargo_path,
            workspace_path,
            features,
            triple,
            child,
            stdout,
            completed: 0,
            total: None,
        })
    }
}

impl Child {
    pub async fn process(&mut self) -> io::Result<Option<Status>> {
        // Unfortunate, but necessary.
        if self.total.is_none() {
            let Self {
                cargo_path,
                workspace_path,
                features,
                triple,
                ..
            } = self;

            let output = Command::new(cargo_path)
                .current_dir(&workspace_path)
                .arg("+nightly")
                .arg("build")
                .arg("--build-plan")
                .arg("--no-default-features")
                .arg("--release")
                .arg("-Zunstable-options")
                .arg(format!("--features={features}"))
                .arg(format!("--target={triple}"))
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .output()
                .await?;

            let build_plan: BuildPlan = serde_json::from_slice(&output.stdout)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

            self.total = Some(build_plan.invocations.len());
        }

        let Some(line) = self.stdout.next_line().await? else {
            return Ok(None);
        };

        let messsge = Message::parse_stream(Cursor::new(line.as_bytes()))
            .next()
            .ok_or_else(invalid_message)?;

        let Self {
            completed, total, ..
        } = self;

        // SAFETY: Total is initialized by the check above.
        let total = unsafe { total.unwrap_unchecked() };

        if *completed < total {
            *completed += 1;
        }

        Ok(Some(Status {
            completed: *completed,
            total,
        }))
    }
}

fn cargo_must_be_an_exe() -> io::Error {
    io::Error::new(
        io::ErrorKind::PermissionDenied,
        "cargo must be an executable",
    )
}

fn missing_stdio() -> io::Error {
    io::Error::new(io::ErrorKind::NotFound, "missing stdio handle")
}

fn invalid_message() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "invalid message from cargo")
}
