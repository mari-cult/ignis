/// Generate target parts.
pub macro generate_target_parts($(
    #[$meta:meta]
    $part:ident/$lower:ident/$assert:ident {
        const LABEL/$host:ident = $label:literal;

        $($value:ident => $key:literal,)*
    }
)*) {$(
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum $part {
        $($value,)*
    }

    impl $part {
        pub(crate) const $host: &'static str = concat!("unknown host ", $label);

        #[must_use]
        pub const fn as_str(&self) -> &'static str {
            match self {
                $(Self::$value => $key,)*
            }
        }

        #[allow(dead_code)]
        #[doc(hidden)]
        #[must_use]
        pub const fn parse($lower: &str) -> Self {
            match Self::try_parse($lower) {
                Some($lower) => $lower,
                None => panic!(concat!("unknown target ", $label)),
            }
        }

        #[must_use]
        pub const fn try_parse($lower: &str) -> Option<Self> {
            let part = match $lower.as_bytes() {
                $(const { $key.as_bytes() } => Self::$value,)*
                _ => return None,
            };

            Some(part)
        }
    }


    impl ::core::fmt::Display for $part {
        #[inline]
        fn fmt(&self, fmt: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            fmt.write_str(self.as_str())
        }
    }

    impl ::core::str::FromStr for $part {
        type Err = $crate::TargetError;

        fn from_str($lower: &str) -> Result<Self, Self::Err> {
            match Self::try_parse($lower) {
                Some($lower) => Ok($lower),
                None => Err($crate::TargetError::$part(Box::from($lower))),
            }
        }
    }

    pub macro $lower($$part:ident) {
        const PART: $crate::$part = $crate::$part::parse(::core::stringify!($$part));

        PART
    }

    pub macro $assert($$string:literal, $$arch:ident) {
        ::core::assert_eq!($$string.parse(), Ok($$crate::$lower!($$arch)));
    }
)*}

generate_target_parts! {
    /// Target architecture.
    Arch/arch/assert_arch {
        const LABEL/UNKNOWN_HOST = "architecture";

        Arm => "arm",
        Arm64 => "arm64",
        X86 => "x86",
        X86_64 => "x86_64",
    }
    /// Target environment (libc).
    Env/env/assert_env {
        const LABEL/UNKNOWN_HOST = "environment";

        Gnu => "gnu",
        Musl => "musl",
    }
    /// Target link method.
    Link/link/assert_link {
        const LABEL/UNKNOWN_HOST = "link target";

        Dynamic => "dynamic",
        Static => "static",
    }
}

impl Arch {
    /// Host architecture.
    // NOTE: tea itself must be compiled for <arch>-musl-static.
    pub const HOST: Self = if cfg!(target_arch = "arm") {
        Self::Arm
    } else if cfg!(target_arch = "arm64") {
        Self::Arm64
    } else if cfg!(target_arch = "x86") {
        Self::X86
    } else if cfg!(target_arch = "x86_64") {
        Self::X86_64
    } else {
        panic!("{}", Self::UNKNOWN_HOST)
    };
}

impl Env {
    /// Host environment.
    // NOTE: tea itself must be compiled for <arch>-musl-static.
    pub const HOST: Self = if cfg!(target_env = "musl") {
        Self::Musl
    } else {
        panic!("{}", Self::UNKNOWN_HOST)
    };
}

impl Link {
    /// Host link target.
    // NOTE: tea itself must be compiled for <arch>-musl-static.
    pub const HOST: Self = if cfg!(target_feature = "crt-static") {
        Self::Static
    } else {
        panic!("{}", Self::UNKNOWN_HOST)
    };
}

#[cfg(test)]
mod tests {
    use super::{assert_arch, assert_env, assert_link};

    #[test]
    fn arch() {
        assert_arch!("arm", arm);
        assert_arch!("arm64", arm64);
        assert_arch!("x86", x86);
        assert_arch!("x86_64", x86_64);
    }

    #[test]
    fn env() {
        assert_env!("gnu", gnu);
        assert_env!("musl", musl);
    }

    #[test]
    fn link() {
        assert_env!("dynamic", dynamic);
        assert_env!("static", static);
    }
}
