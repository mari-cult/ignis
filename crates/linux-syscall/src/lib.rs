//! Linux system calls.

#![cfg_attr(not(any(doc, feature = "std")), no_std)]

pub use self::{
    error::Error,
    id::Id,
    traits::{Arg, FromOutput, IntoArg},
};

pub(crate) use macros::unreachable;

mod error;
mod id;
mod macros;
mod traits;
