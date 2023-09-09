use crate::{macros::unreachable, Error};

/// A raw system call argument.
pub trait Arg: sealed::Sealed {
    #[doc(hidden)]
    fn to_usize(self) -> usize;
}

/// A trait for converting into a raw system call argument.
pub trait IntoArg {
    /// Raw system call argument.
    type Target: Arg;

    /// Perform the conversion.
    fn into_arg(self) -> Self::Target;
}

/// A trait for decoding system call return values.
pub trait FromOutput {
    /// Perform the conversion.
    fn from_output<A: Arg>(ouput: A) -> Self;
}

mod sealed {
    pub trait Sealed {}
}

macro_rules! ptr {
    ($generic:ident: $ty:ty) => {
        impl<$generic> sealed::Sealed for $ty {}
        impl<$generic> Arg for $ty {
            fn to_usize(self) -> usize {
                // FIXME: Use `self.addr()` when it's stablized.
                self as usize
            }
        }

        impl<$generic> IntoArg for $ty {
            type Target = $ty;

            fn into_arg(self) -> Self::Target {
                self
            }
        }
    };
}

macro_rules! into_args_usize {
    ($($ty:ty,)*) => {$(
        impl IntoArg for $ty {
            type Target = usize;

            fn into_arg(self) -> Self::Target {
                self as usize
            }
        }
    )*};
}

impl sealed::Sealed for usize {}
impl Arg for usize {
    fn to_usize(self) -> usize {
        self
    }
}

ptr!(T: *const T);
ptr!(T: *mut T);

into_args_usize! {
    u8, u16, u32, u64, usize,
    i8, i16, i32, i64, isize,
}

impl FromOutput for Result<usize, u16> {
    fn from_output<A: Arg>(output: A) -> Self {
        let output = output.to_usize();
        let error = -(output as isize) as usize;

        if (1..4096).contains(&error) {
            Err(error as u16)
        } else {
            Ok(output)
        }
    }
}

impl FromOutput for Result<usize, Error> {
    fn from_output<A: Arg>(output: A) -> Self {
        Result::<usize, u16>::from_output(output)
            .map_err(|error| Error::from_raw_os_error(error as i32))
    }
}

impl FromOutput for Result<(), Error> {
    fn from_output<A: Arg>(output: A) -> Self {
        Result::<usize, Error>::from_output(output).map(|_value| ())
    }
}

impl FromOutput for Error {
    fn from_output<A: Arg>(output: A) -> Self {
        let output = Result::<usize, Error>::from_output(output);

        match output {
            Ok(_value) => unreachable!(),
            Err(error) => error,
        }
    }
}

#[allow(unused_macros)]
macro_rules! from_output_map_err {
    ($($ty:ty),*) => {$(
        impl FromOutput for Result<usize, $ty> {
            fn from_output<A: Arg>(output: A) -> Self {
                Result::<usize, Error>::from_output(output).map_err(Into::into)
            }
        }

        impl FromOutput for Result<(), $ty> {
            fn from_output<A: Arg>(output: A) -> Self {
                Result::<usize, $ty>::from_output(output)?;

                Ok(())
            }
        }

        impl FromOutput for $ty {
            fn from_output<A: Arg>(output: A) -> Self {
                Error::from_output(output).into()
            }
        }
    )*};
}

#[cfg(feature = "nix")]
from_output_map_err!(nix::errno::Errno);

#[cfg(feature = "rustix")]
from_output_map_err!(rustix::io::Errno);

#[cfg(feature = "std")]
from_output_map_err!(std::io::ErrorKind, std::io::Error);

#[cfg(feature = "std")]
mod impl_std {
    use {
        super::*,
        std::{
            io,
            os::unix::io::{FromRawFd, OwnedFd, RawFd},
        },
    };

    impl FromOutput for io::Result<OwnedFd> {
        fn from_output<A: Arg>(output: A) -> Self {
            io::Result::<usize>::from_output(output)
                .map(|fd| unsafe { OwnedFd::from_raw_fd(fd as RawFd) })
        }
    }
}
