#![deny(warnings)]
#![allow(incomplete_features)]
#![allow(unused_macros)]
#![feature(const_trait_impl)]
#![feature(decl_macro)]
#![feature(inline_const_pat)]
#![feature(macro_metavar_expr)]

use {
    std::{fmt, str::FromStr},
    thiserror::Error,
};

pub use crate::part::{arch, env, link, Arch, Env, Link};

mod part;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum TargetError {
    #[error("unknown target architecture: {0}")]
    Arch(Box<str>),

    #[error("unknown target environment: {0}")]
    Env(Box<str>),

    #[error("unknown target link target: {0}")]
    Link(Box<str>),

    #[error("the gnu environment does not support static linking")]
    GnuStaticUnsupported,

    #[error("invalid target: {0}")]
    Invalid(Box<str>),
}

/// Install target.
///
/// Target string mapping:
///
/// ```rust
/// "zstd" -> HOST
/// "zstd@gnu" -> (HOST, Gnu, HOST)
/// "zstd@gnu-static" -> GnuStaticUnsupported
/// "zstd@arm64" -> (Arm64, HOST, HOST)
/// "zstd@arm64-musl" -> (Arm64, Musl, HOST)
/// "zstd@arm64-musl-dynamic" -> (Arm64, Musl, Dynamic)
/// ```
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Target {
    arch: Arch,
    env: Env,
    link: Link,
}

impl Target {
    /// The host target.
    pub const HOST: Target = Target::new_host(Arch::HOST, Env::HOST, Link::HOST);

    /// Create a new `Target`.
    pub const fn new(arch: Arch, env: Env, link: Link) -> Result<Self, TargetError> {
        if matches!((env, link), (Env::Gnu, Link::Static)) {
            return Err(TargetError::GnuStaticUnsupported);
        }

        Ok(Self { arch, env, link })
    }

    const fn new_host(arch: Arch, env: Env, link: Link) -> Self {
        if matches!((env, link), (Env::Gnu, Link::Static)) {
            panic!("the gnu environment does not support static linking");
        }

        Self { arch, env, link }
    }
}

impl FromStr for Target {
    type Err = TargetError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let mut iter = string.splitn(3, '-');

        match (iter.next(), iter.next(), iter.next()) {
            (Some(arch), Some(env), Some(link)) => {
                Self::new(arch.parse()?, env.parse()?, link.parse()?)
            }
            (Some(opaque_a), Some(opaque_b), None) => {
                if let (Ok(arch), Ok(env)) = (opaque_a.parse(), opaque_b.parse()) {
                    // NOTE: For convenience, assume `gnu-dynamic`.
                    let link = if env == Env::Gnu {
                        Link::Dynamic
                    } else {
                        Link::HOST
                    };

                    Self::new(arch, env, link)
                } else if let (Ok(env), Ok(link)) = (opaque_a.parse(), opaque_b.parse()) {
                    Self::new(Arch::HOST, env, link)
                } else if let (Ok(arch), Ok(link)) = (opaque_a.parse(), opaque_b.parse()) {
                    Self::new(arch, Env::HOST, link)
                } else {
                    Err(TargetError::Invalid(Box::from(string)))
                }
            }
            (Some(opaque), None, None) => {
                if let Ok(arch) = opaque.parse() {
                    Self::new(arch, Env::HOST, Link::HOST)
                } else if let Ok(env) = opaque.parse() {
                    // NOTE: For convenience, assume `gnu-dynamic`.
                    let link = if env == Env::Gnu {
                        Link::Dynamic
                    } else {
                        Link::HOST
                    };

                    Self::new(Arch::HOST, env, link)
                } else if let Ok(link) = opaque.parse() {
                    Self::new(Arch::HOST, Env::HOST, link)
                } else {
                    Err(TargetError::Invalid(Box::from(string)))
                }
            }
            _ => panic!(),
        }
    }
}

impl fmt::Debug for Target {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, fmt)
    }
}

impl fmt::Display for Target {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { arch, env, link } = self;

        write!(fmt, "{arch}-{env}-{link}")
    }
}

pub macro target($arch:ident-$env:ident-$link:ident) {
    const TARGET: $crate::Target = $crate::Target {
        arch: $crate::arch!($arch),
        env: $crate::env!($env),
        link: $crate::link!($link),
    };

    TARGET
}
