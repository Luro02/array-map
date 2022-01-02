#![cfg_attr(not(feature = "std"), no_std)]
#![feature(
    array_methods,
    never_type,
    maybe_uninit_uninit_array,
    maybe_uninit_array_assume_init,
    try_trait_v2,
    stmt_expr_attributes,
    generic_associated_types
)]
#![cfg_attr(feature = "nightly", feature(core_intrinsics))]
#![warn(clippy::nursery, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::module_inception,
    clippy::redundant_pub_crate
)]

mod errors;
pub mod ext;
mod external_trait_impls;
pub mod map;
mod raw;
pub mod set;
mod utils;

pub use crate::map::{ArrayMap, ArrayMapFacade, DefaultHashBuilder, IndexMap};
pub use errors::*;

#[cfg(feature = "alloc")]
extern crate alloc;
