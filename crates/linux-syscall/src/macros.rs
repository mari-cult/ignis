#[cfg(target_arch = "aarch64")]
#[doc(hidden)]
#[macro_export]
macro_rules! syscall_inner {
    ($id:ident, $($arg0:expr $(, $arg1:expr $(, $arg2:expr $(, $arg3:expr $(, $arg4:expr $(, $arg5:expr $(,)?)?)?)?)?)?)?) => {{
        let output: usize;

        ::core::arch::asm!(
            "svc 0",
            in("x8") $crate::Id::$id as usize,
            $(inout("x0") $crate::IntoArg::into_arg($arg0) => output,
            $(in("x1") $crate::IntoArg::into_arg($arg1),
            $(in("x2") $crate::IntoArg::into_arg($arg2),
            $(in("x3") $crate::IntoArg::into_arg($arg3),
            $(in("x4") $crate::IntoArg::into_arg($arg4),
            $(in("x5") $crate::IntoArg::into_arg($arg5),
            )?)?)?)?)?)?
            options(nostack),
        );

        $crate::FromOutput::from_output(output)
    }};
    (noreturn; $id:ident, $($arg0:expr $(, $arg1:expr $(, $arg2:expr $(, $arg3:expr $(, $arg4:expr $(, $arg5:expr $(,)?)?)?)?)?)?)?) => {{
        ::core::arch::asm!(
            "svc 0",
            in("x8") $crate::Id::$id as usize,
            $(in("x0") $crate::IntoArg::into_arg($arg0),
            $(in("x1") $crate::IntoArg::into_arg($arg1),
            $(in("x2") $crate::IntoArg::into_arg($arg2),
            $(in("x3") $crate::IntoArg::into_arg($arg3),
            $(in("x4") $crate::IntoArg::into_arg($arg4),
            $(in("x5") $crate::IntoArg::into_arg($arg5),
            )?)?)?)?)?)?
            options(noreturn, nostack),
        );
    }};
}

#[cfg(target_arch = "x86_64")]
#[doc(hidden)]
#[macro_export]
macro_rules! syscall_inner {
    ($id:ident, $($arg0:expr $(, $arg1:expr $(, $arg2:expr $(, $arg3:expr $(, $arg4:expr $(, $arg5:expr $(,)?)?)?)?)?)?)?) => {{
        let output: usize;

        ::core::arch::asm!(
            "syscall",
            inout("rax") $crate::Id::$id as usize => output,
            $(in("rdi") $crate::IntoArg::into_arg($arg0),
            $(in("rsi") $crate::IntoArg::into_arg($arg1),
            $(in("rdx") $crate::IntoArg::into_arg($arg2),
            $(in("r10") $crate::IntoArg::into_arg($arg3),
            $(in("r8") $crate::IntoArg::into_arg($arg4),
            $(in("r9") $crate::IntoArg::into_arg($arg5),
            )?)?)?)?)?)?
            out("r11") _,
            out("rcx") _,
            options(nostack),
        );

        $crate::FromOutput::from_output(output)
    }};
    (noreturn; $id:ident, $($arg0:expr $(, $arg1:expr $(, $arg2:expr $(, $arg3:expr $(, $arg4:expr $(, $arg5:expr $(,)?)?)?)?)?)?)?) => {{
        ::core::arch::asm!(
            "syscall",
            in("rax") $crate::Id::$id as usize,
            $(in("rdi") $crate::IntoArg::into_arg($arg0),
            $(in("rsi") $crate::IntoArg::into_arg($arg1),
            $(in("rdx") $crate::IntoArg::into_arg($arg2),
            $(in("r10") $crate::IntoArg::into_arg($arg3),
            $(in("r8") $crate::IntoArg::into_arg($arg4),
            $(in("r9") $crate::IntoArg::into_arg($arg5),
            )?)?)?)?)?)?
            options(noreturn, nostack),
        );
    }};
}

/// Perform a system call.
#[macro_export]
macro_rules! syscall {
    ($($tt:tt)*) => {
        $crate::syscall_inner!($($tt)*)
    };
}

macro_rules! ids {
    ($($variant:ident = $value:ident,)*) => {
        /// System call ID.
        #[allow(missing_docs)]
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
        #[repr(u32)]
        pub enum Id {
            $($variant = linux_raw_sys::general::$value,)*
        }

        impl Id {
            /// Obtain a system call ID from the raw representation.
            pub const fn from_raw(id: u32) -> Option<Self> {
                let id = match id {
                    $(linux_raw_sys::general::$value => Self::$variant,)*
                    _ => return None,
                };

                Some(id)
            }

            /// Convert to the raw representation.
            pub const fn to_raw(self) -> u32 {
                self as u32
            }
        }
    };
}

macro_rules! unreachable {
    () => {{
        #[cfg(debug_assertions)]
        ::core::unreachable!();

        #[cfg(not(debug_assertions))]
        unsafe {
            ::core::hint::unreachable_unchecked()
        };
    }};
}

pub(crate) use {ids, unreachable};
