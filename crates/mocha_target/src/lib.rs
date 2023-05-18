#![deny(warnings)]

use {
    std::{fmt, str::FromStr},
    thiserror::Error,
};

pub use crate::part::{Arch, Env, Link};

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

    /// Return the equivalent rust target triple.
    // NOTE: `rustc --print target-list`
    pub fn rust_triple(&self) -> &'static str {
        let Self { arch, env, .. } = self;

        match (arch, env) {
            // gnu
            (Arch::Arm, Env::Gnu) => "armv7-unknown-linux-gnueabi",
            (Arch::Arm64, Env::Gnu) => "aarch64-unknown-linux-gnu",
            (Arch::X86, Env::Gnu) => "i686-unknown-linux-gnu",
            (Arch::X86_64, Env::Gnu) => "x86_64-unknown-linux-gnu",
            // musl
            (Arch::Arm, Env::Musl) => "armv7-unknown-linux-musleabi",
            (Arch::Arm64, Env::Musl) => "aarch64-unknown-linux-musl",
            (Arch::X86, Env::Musl) => "i686-unknown-linux-musl",
            (Arch::X86_64, Env::Musl) => "x86_64-unknown-linux-musl",
        }
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
