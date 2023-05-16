/// Generate target parts.
macro_rules! generate_target_parts {
    (
        $(
            #[$meta:meta]
            $part:ident/$lower:ident/$assert:ident {
                const LABEL = $label:literal;

                $($value:ident => $key:literal,)*
            }
        )*
    ) => {
        $(
            #[derive(Clone, Copy, Debug, Eq, PartialEq)]
            pub enum $part {
                $($value,)*
            }

            impl $part {
                #[must_use]
                pub const fn as_str(&self) -> &'static str {
                    match self {
                        $(Self::$value => $key,)*
                    }
                }

                #[allow(dead_code)]
                #[doc(hidden)]
                #[must_use]
                pub fn parse($lower: &str) -> Self {
                    match Self::try_parse($lower) {
                        Some($lower) => $lower,
                        None => panic!(concat!("unknown target ", $label)),
                    }
                }

                #[must_use]
                pub fn try_parse($lower: &str) -> Option<Self> {
                    let part = match $lower {
                        $($key => Self::$value,)*
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
        )*
    }
}

generate_target_parts! {
    /// Target architecture.
    Arch/arch/assert_arch {
        const LABEL = "architecture";

        Arm => "arm",
        Arm64 => "arm64",
        X86 => "x86",
        X86_64 => "x86_64",
    }
    /// Target environment (libc).
    Env/env/assert_env {
        const LABEL = "environment";

        Gnu => "gnu",
        Musl => "musl",
    }
    /// Target link method.
    Link/link/assert_link {
        const LABEL = "link target";

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
        panic!("Unsupported architecture, please create an issue to add support.");
    };
}

impl Env {
    /// The target host environment.
    ///
    /// This value is **always** `Env::Musl`.
    pub const HOST: Self = Self::Musl;
}

impl Link {
    /// The target link type.
    ///
    /// This value is **always** `Link::Static`.
    pub const HOST: Self = Self::Static;
}
