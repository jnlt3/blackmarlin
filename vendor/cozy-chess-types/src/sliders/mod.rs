// This module is never exposed by the main crate.
#![allow(missing_docs)]

mod common;

#[cfg(not(feature = "pext"))]
mod magic;
#[cfg(feature = "pext")]
mod pext;

pub use common::*;

#[cfg(not(feature = "pext"))]
pub use magic::*;
#[cfg(feature = "pext")]
pub use pext::*;
