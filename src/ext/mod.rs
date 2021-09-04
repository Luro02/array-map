//! This module contains extension traits that are not yet present in the
//! standard libary.
mod to_iter;
mod try_extend;
mod try_from_iterator;

pub use to_iter::*;
pub use try_extend::*;
pub use try_from_iterator::*;
