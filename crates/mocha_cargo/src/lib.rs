#![deny(warnings)]

use {
    camino::{Utf8Path, Utf8PathBuf},
    cargo_metadata::Message,
    mocha_target::Target,
    mocha_utils::{Category, Command, Rule},
    serde::Deserialize,
    std::{
        collections::BTreeSet,
        fmt, fs,
        future::Future,
        io::{self, Cursor},
        os::unix::fs::PermissionsExt,
        pin::Pin,
        process::Stdio,
        ptr,
        task::{Context, Poll},
    },
    tokio::{
        io::{AsyncBufReadExt, BufReader, Lines},
        process::ChildStdout,
    },
};

mod error;

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

enum ChildState {
    Plan {
        output: Pin<Box<dyn Future<Output = io::Result<mocha_utils::Output>>>>,
        child: mocha_utils::Child,
        stdout: Lines<BufReader<ChildStdout>>,
    },
    Build {
        child: mocha_utils::Child,
        stdout: Lines<BufReader<ChildStdout>>,
        completed: usize,
        total: usize,
    },
    Error,
    Done,
}

/// A `cargo build` child process.
pub struct Child {
    state: ChildState,
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
            return Err(error::must_be_an_exe());
        }

        // Is not executable by the user.
        if metadata.permissions().mode() & 0o100 != 0o100 {
            return Err(error::must_be_an_exe());
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
        let triple = target.rust_triple();

        let output = Command::new(&cargo_path)
            .current_dir(&workspace_path)
            .arg("+nightly")
            // `zigbuild` doesn't support `--build-plan`.
            .arg("build")
            .arg("--build-plan")
            .arg("--no-default-features")
            .arg("--release")
            .arg("-Zunstable-options")
            .arg(format!("--features={features}"))
            .arg(format!("--target={triple}"))
            .execution_policy((Category::SetUsers, Rule::Kill))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output();

        let output = Box::pin(output);

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
            .execution_policy((Category::SetUsers, Rule::Kill))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let stdout = child.stdout().ok_or_else(error::missing_stdio)?;
        let stdout = BufReader::new(stdout).lines();

        Ok(Child {
            state: ChildState::Plan {
                output,
                child,
                stdout,
            },
        })
    }
}

impl Future for ChildState {
    type Output = io::Result<Option<(usize, usize)>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Ceeating a bitwise-copy is necessary to advance the state machine.
        let (state, poll) = match unsafe { ptr::read(self.as_mut().get_unchecked_mut()) } {
            ChildState::Plan {
                mut output,
                child,
                stdout,
            } => {
                let poll =
                    unsafe { Pin::new_unchecked(&mut output) }
                        .poll(cx)
                        .map(|maybe_output| {
                            maybe_output.and_then(|output| parse_total(&output.stdout))
                        });

                match poll {
                    Poll::Ready(result) => result
                        .map(|total| {
                            let state = ChildState::Build {
                                child,
                                stdout,
                                completed: 0,
                                total,
                            };

                            (state, Poll::Pending)
                        })
                        .unwrap_or_else(|error| (ChildState::Error, Poll::Ready(Err(error)))),

                    Poll::Pending => {
                        let state = ChildState::Plan {
                            output,
                            child,
                            stdout,
                        };

                        (state, Poll::Pending)
                    }
                }
            }
            ChildState::Build {
                child,
                mut stdout,
                mut completed,
                total,
            } => {
                let poll = unsafe { Pin::new_unchecked(&mut stdout) }
                    .poll_next_line(cx)
                    .map(|result| {
                        result
                            .transpose()
                            .map(|maybe_line| maybe_line.map(|line| parse_message(line.as_bytes())))
                    });

                match poll {
                    Poll::Ready(Some(_maybe_message)) => {
                        completed += 1;
                        let state = ChildState::Build {
                            child,
                            stdout,
                            completed,
                            total,
                        };

                        (state, Poll::Ready(Ok(Some((completed, total)))))
                    }
                    Poll::Ready(None) => (ChildState::Done, Poll::Ready(Ok(None))),
                    Poll::Pending => {
                        let state = ChildState::Build {
                            child,
                            stdout,
                            completed,
                            total,
                        };

                        (state, Poll::Pending)
                    }
                }
            }
            ChildState::Error => (ChildState::Error, Poll::Ready(Err(error::already_exited()))),
            ChildState::Done => (ChildState::Done, Poll::Ready(Ok(None))),
        };

        // Advance the state machine.
        unsafe {
            ptr::write(self.as_mut().get_unchecked_mut(), state);
        }

        poll
    }
}

impl Child {
    pub async fn process(&mut self) -> io::Result<Option<(usize, usize)>> {
        Pin::new(&mut self.state).await
    }
}

impl fmt::Debug for ChildState {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChildState::Plan { child, stdout, .. } => fmt
                .debug_struct("Plan")
                .field("output", &"dyn Future")
                .field("child", child)
                .field("stdout", stdout)
                .finish(),
            ChildState::Build {
                child,
                stdout,
                completed,
                total,
            } => fmt
                .debug_struct("Build")
                .field("child", child)
                .field("stdout", stdout)
                .field("completed", completed)
                .field("total", total)
                .finish(),
            ChildState::Error => fmt.debug_struct("Error").finish(),
            ChildState::Done => fmt.debug_struct("Done").finish(),
        }
    }
}

fn parse_total(bytes: &[u8]) -> io::Result<usize> {
    let build_plan: BuildPlan = serde_json::from_slice(bytes).map_err(error::invalid_plan)?;

    Ok(build_plan.invocations.len())
}

fn parse_message(bytes: &[u8]) -> io::Result<Message> {
    let message = Message::parse_stream(Cursor::new(bytes))
        .next()
        .ok_or_else(error::invalid_message)??;

    Ok(message)
}
