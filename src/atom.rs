use {
    milk_target::{Target, TargetError},
    std::{fmt, str::FromStr},
};

#[derive(Clone, Eq, PartialEq)]
pub struct Atom {
    pub package: String,
    pub target: Target,
}

impl FromStr for Atom {
    type Err = TargetError;

    fn from_str(atom: &str) -> Result<Self, Self::Err> {
        if atom.is_empty() {
            Err(TargetError::Invalid(Box::from("empty atom")))
        } else {
            let (package, target) = match atom.split_once('@') {
                Some((package, target)) => (package, target.parse()?),
                None => (atom, Target::HOST),
            };

            Ok(Self {
                package: String::from(package),
                target,
            })
        }
    }
}

impl fmt::Debug for Atom {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, fmt)
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { package, target } = self;

        write!(fmt, "{package}@{target}")
    }
}
