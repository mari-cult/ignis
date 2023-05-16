#![deny(warnings)]

use {
    camino::Utf8PathBuf,
    std::{
        collections::HashMap,
        ffi::OsString,
        fmt, io,
        os::unix::{ffi::OsStringExt, process::CommandExt},
        process::{Child as StdChild, Command as StdCommand, ExitStatus},
    },
    tokio::process::{Child as TokioChild, Command as TokioCommand},
};

pub use policy::{Category, Policy, Rule};

mod policy;
mod sys;

/// A strict version of [`Command`](std::process::Command),
/// with various extra features.
pub struct Command {
    program: Utf8PathBuf,
    args: Vec<Vec<u8>>,
    envs: HashMap<String, Vec<u8>>,
    current_dir: Option<Utf8PathBuf>,
    user_id: Option<u32>,
    group_id: Option<u32>,
    group_ids: Vec<u32>,
    execution_policy: Policy,
}

/// Representation of a child process spawned onto an event loop.
pub struct Child {
    child: TokioChild,
}

/// Representation of a running or exited child process.
pub struct BlockingChild {
    child: StdChild,
}

impl Command {
    /// Create a new `Command` for executing `program`.
    ///
    /// `program` must be a canonical path.
    pub fn new<P>(program: P) -> Self
    where
        P: Into<Utf8PathBuf>,
    {
        Self {
            program: program.into(),
            args: Vec::new(),
            envs: HashMap::new(),
            current_dir: None,
            user_id: None,
            group_id: None,
            group_ids: Vec::new(),
            execution_policy: Policy::new(),
        }
    }

    /// Adds an argument to pass to the program.
    #[inline]
    pub fn arg<A>(mut self, arg: A) -> Self
    where
        A: Into<Vec<u8>>,
    {
        self.args.push(arg.into());
        self
    }

    /// Adds multiple arguments to pass to the program.
    #[inline]
    pub fn args<I, A>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = A>,
        A: Into<Vec<u8>>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    /// Insert or update an environment mapping.
    #[inline]
    pub fn env<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<Vec<u8>>,
    {
        self.envs.insert(key.into(), value.into());
        self
    }

    /// Insert or update multiple environment mappings.
    #[inline]
    pub fn envs<I, K, V>(mut self, envs: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<Vec<u8>>,
    {
        self.envs.extend(
            envs.into_iter()
                .map(|(key, value)| (key.into(), value.into())),
        );

        self
    }

    /// Set the working directory of the child process.
    #[inline]
    pub fn current_dir<D>(mut self, current_dir: D) -> Self
    where
        D: Into<Utf8PathBuf>,
    {
        self.current_dir = Some(current_dir.into());
        self
    }

    /// Set the user ID of the child process.
    #[inline]
    pub fn user_id(mut self, id: u32) -> Self {
        self.user_id = Some(id);
        self
    }

    /// Set the group ID of the child process.
    #[inline]
    pub fn group_id(mut self, id: u32) -> Self {
        self.group_id = Some(id);
        self
    }

    /// Set the group IDs of the child process.
    #[inline]
    pub fn group_ids<I>(mut self, ids: I) -> Self
    where
        I: IntoIterator<Item = u32>,
    {
        self.group_ids.extend(ids);
        self
    }

    /// Configure the execution policy for this Command.
    #[inline]
    pub fn execution_policy<P>(mut self, policy: P) -> Self
    where
        P: Into<Policy>,
    {
        self.execution_policy = self.execution_policy.and(policy.into());
        self
    }

    /// Convert `Command` into an `std::process::Command`.
    fn into_command(self) -> StdCommand {
        let Self {
            program,
            args,
            envs,
            current_dir,
            user_id,
            group_id,
            group_ids,
            execution_policy,
        } = self;

        let args = args.into_iter().map(OsString::from_vec);
        let envs = envs
            .into_iter()
            .map(|(key, value)| (OsString::from(key), OsString::from_vec(value)));

        let mut command = StdCommand::new(program);

        command.args(args).envs(envs);

        if let Some(current_dir) = current_dir {
            command.current_dir(current_dir);
        }

        unsafe {
            command.pre_exec(move || {
                // IDs must be set **before** SecComp policy is installed, to avoid blocking it in the case of `Category::SetUsers`.
                sys::set_ids(user_id, group_id, group_ids.clone())?;
                policy::set_current_policy(&execution_policy)?;

                Ok(())
            });
        }

        command
    }

    /// Replace the current process with this one.
    #[inline]
    pub fn spawn_in_place(self) -> io::Error {
        self.into_command().exec()
    }

    /// Executes the command as a child process, returning a handle to it.
    #[inline]
    pub fn spawn(self) -> io::Result<Child> {
        let child = TokioCommand::from(self.into_command()).spawn()?;

        Ok(Child { child })
    }

    /// Executes the command as a child process, returning a handle to it.
    #[inline]
    pub fn spawn_blocking(self) -> io::Result<BlockingChild> {
        let child = self.into_command().spawn()?;

        Ok(BlockingChild { child })
    }
}

impl Child {
    /// Returns the OS-assigned process identifier associated with this child, while it is still running.
    pub fn id(&self) -> Option<u32> {
        self.child.id()
    }

    /// Forces the child to exit.
    pub async fn kill(&mut self) -> io::Result<()> {
        self.child.kill().await
    }

    /// Wait for the child process to exit completely, returnin the status that it exited with.
    pub async fn wait(&mut self) -> io::Result<ExitStatus> {
        self.child.wait().await
    }
}

impl BlockingChild {
    /// Returns the OS-assigned process identifier associated with this child, while it is still running.
    pub fn id(&self) -> Option<u32> {
        Some(self.child.id())
    }

    /// Forces the child process to exit.
    pub fn kill(&mut self) -> io::Result<()> {
        self.child.kill()
    }

    /// Wait for the child process to exit completely, returnin the status that it exited with.
    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        self.child.wait()
    }
}

impl fmt::Debug for Command {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Command").finish_non_exhaustive()
    }
}

impl fmt::Debug for Child {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Child").finish_non_exhaustive()
    }
}

impl fmt::Debug for BlockingChild {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("BlockingChild").finish_non_exhaustive()
    }
}
