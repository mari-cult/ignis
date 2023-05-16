use {
    sc::nr,
    std::{collections::HashMap, io},
};

/// System call.
#[derive(Clone, Copy, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
#[repr(usize)]
pub enum Call {
    GetUid = nr::GETUID,
    GetGid = nr::GETGID,
    GetGroups = nr::GETGROUPS,

    GetEUid = nr::GETEUID,
    GetEGid = nr::GETEGID,
}

/// SecComp rule.
///
/// The default is allow.
#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq)]
pub enum Rule {
    #[default]
    Allow,
    Kill,
    Error(io::ErrorKind),
}

/// SecComp policy.
///
/// The default for all system calls are allowed.
#[derive(Clone, Default, PartialEq)]
#[must_use = "You must apply a SecComp policy to a Command to use it."]
pub struct Policy {
    default_rule: Rule,
    rules: HashMap<Call, Rule>,
}

impl Policy {
    /// All system calls are allowed.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// All system calls will kill the process.
    #[inline]
    pub fn kill() -> Self {
        Self {
            default_rule: Rule::Kill,
            ..Self::new()
        }
    }

    /// Set the rule for a particular system call.
    #[inline]
    pub fn with<R: Into<Rule>>(mut self, call: Call, rule: R) -> Self {
        self.rules.insert(call, rule.into());
        self
    }

    /// Set the SecComp policy of the current thread.
    ///
    /// # Safety
    ///
    /// As SecComp controls how system calls are handled, this may have
    /// unintended side-effects on the current thread's behaviour.
    pub(crate) unsafe fn set_policy(&self) -> io::Result<()> {
        let Self {
            default_rule,
            rules,
        } = self;

        let mut context = seccomp::Context::default(map_rule(*default_rule)).map_err(map_error)?;

        for (call, rule) in rules {
            let comparsion = seccomp::Compare::arg(0)
                .using(seccomp::Op::Gt)
                .with(0)
                .build()
                .unwrap();

            let rule = seccomp::Rule::new(*call as usize, comparsion, map_rule(*rule));

            context.add_rule(rule).map_err(map_error)?;
        }

        context.load().map_err(map_error)?;

        Ok(())
    }
}

impl From<io::ErrorKind> for Rule {
    #[inline]
    fn from(error: io::ErrorKind) -> Self {
        Rule::Error(error)
    }
}

/// Map a `Rule` to a `seccomp::Action`.
fn map_rule(rule: Rule) -> seccomp::Action {
    match rule {
        Rule::Allow => seccomp::Action::Allow,
        Rule::Kill => seccomp::Action::KillProcess,
        Rule::Error(error) => seccomp::Action::Errno(
            io::Error::new(error, "seccomp")
                .raw_os_error()
                .unwrap_or_default(),
        ),
    }
}

/// Map a `seccomp::SeccompError` to an `io::Error`.
fn map_error(error: seccomp::SeccompError) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, error)
}

fn main() {
    use std::{os::unix::process::CommandExt, process::Command};

    let policy = Policy::new()
        .with(Call::GetUid, io::ErrorKind::PermissionDenied)
        .with(Call::GetGid, io::ErrorKind::PermissionDenied)
        .with(Call::GetGroups, io::ErrorKind::PermissionDenied)
        .with(Call::GetEUid, io::ErrorKind::PermissionDenied)
        .with(Call::GetEGid, io::ErrorKind::PermissionDenied);

    unsafe {
        Command::new("id")
            .pre_exec(move || policy.set_policy())
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }
}