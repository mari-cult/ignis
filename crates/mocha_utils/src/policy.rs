use {sc::nr, std::io};

/// Execution policy rule.
#[derive(Clone, Copy, Debug, Default)]
pub enum Rule {
    /// Allow (default).
    #[default]
    Allow,

    /// Return an error code.
    Error(io::ErrorKind),

    /// Halt with a code.
    Halt(u32),

    /// Kill the process.
    Kill,
}

/// Execution policy category.
#[derive(Clone, Copy, Debug)]
pub enum Category {
    Network,
    Users,
    SetUsers,
}

/// Execution policy for [`Command`](crate::Command).
#[derive(Default)]
pub struct Policy {
    network_rule: Option<Rule>,
    users_rule: Option<Rule>,
    set_users_rule: Option<Rule>,
}

impl Policy {
    /// Default policy (allow all).
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Combine polices.
    #[inline]
    pub(crate) fn and(self, other: Self) -> Self {
        Self {
            network_rule: self.network_rule.or(other.network_rule),
            users_rule: self.users_rule.or(other.users_rule),
            set_users_rule: self.set_users_rule.or(other.set_users_rule),
        }
    }
}

impl From<io::ErrorKind> for Rule {
    fn from(error: io::ErrorKind) -> Self {
        Self::Error(error)
    }
}

impl From<(Category, Rule)> for Policy {
    fn from((category, rule): (Category, Rule)) -> Self {
        let mut policy = Self::new();

        match category {
            Category::Network => policy.network_rule = Some(rule),
            Category::Users => policy.users_rule = Some(rule),
            Category::SetUsers => policy.set_users_rule = Some(rule),
        }

        policy
    }
}

impl From<(Category, io::ErrorKind)> for Policy {
    fn from((category, rule): (Category, io::ErrorKind)) -> Self {
        Policy::from((category, Rule::from(rule)))
    }
}

/// Convert `seccomp::SeccompError` to `io::Error`.
#[inline]
fn into_io(error: seccomp::SeccompError) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, error)
}

/// Convert `Rule` to `seccomp::Action`.
#[inline]
fn into_action(rule: Rule) -> seccomp::Action {
    use seccomp::Action;

    match rule {
        Rule::Allow => Action::Allow,
        Rule::Error(kind) => {
            // Extract raw OS code from `io::ErrorKind`.
            let error = io::Error::new(kind, "");
            let code = error.raw_os_error().unwrap_or_default();

            Action::Errno(code)
        }
        Rule::Halt(code) => Action::Trace(code),
        Rule::Kill => Action::KillProcess,
    }
}

/// Set the SecComp policy of the current thread.
///
/// # Safety
///
/// As SecComp controls how system calls are handled, this may have
/// unintended side-effects on the current thread's behaviour.
pub(crate) unsafe fn set_current_policy(policy: &Policy) -> io::Result<()> {
    use seccomp::{Action, Context};

    let mut context = Context::default(Action::Allow).map_err(into_io)?;

    let Policy {
        network_rule,
        users_rule,
        set_users_rule,
    } = policy;

    add_rules(
        &mut context,
        &[nr::ACCEPT, nr::ACCEPT4, nr::SOCKET, nr::SOCKETPAIR],
        network_rule,
    )?;

    add_rules(
        &mut context,
        &[nr::GETUID, nr::GETGID, nr::GETGROUPS],
        users_rule,
    )?;

    add_rules(
        &mut context,
        &[nr::SETUID, nr::SETGID, nr::SETGROUPS],
        set_users_rule,
    )?;

    context.load().map_err(into_io)?;

    Ok(())
}

/// Add a set of system calls with the same rule to the SecComp policy.
fn add_rules(
    context: &mut seccomp::Context,
    calls: &[usize],
    rule: &Option<Rule>,
) -> io::Result<()> {
    use seccomp::{Compare, Op};

    let action = match *rule {
        // If allow, or not present, allow anyway, it is the default behaviour.
        Some(Rule::Allow) | None => return Ok(()),
        Some(rule) => into_action(rule),
    };

    for call in calls {
        context
            .add_rule(seccomp::Rule::new(
                *call,
                Compare::arg(0).with(0).using(Op::Gt).build().unwrap(),
                action,
            ))
            .map_err(into_io)?;
    }

    Ok(())
}
