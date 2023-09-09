//! Linux system calls.

#![deny(invalid_reference_casting)]
#![deny(missing_docs)]
#![deny(warnings)]
#![cfg_attr(not(any(doc, feature = "std")), no_std)]

pub use self::{
    error::Error,
    id::Id,
    traits::{Arg, FromOutput, IntoArg},
};

mod error;
mod id;
mod macros;
mod traits;
