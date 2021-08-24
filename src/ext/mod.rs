//! This module contains extension traits that are not yet present in the
//! standard libary.
mod into_immutable_iter;
mod try_extend;
mod try_from_iterator;

pub use into_immutable_iter::*;
pub use try_extend::*;
pub use try_from_iterator::*;
