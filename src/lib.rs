#![cfg_attr(not(feature = "std"), no_std)]
#![feature(
    const_generics,
    array_methods,
    never_type,
    maybe_uninit_uninit_array,
    maybe_uninit_array_assume_init,
    try_trait_v2,
    stmt_expr_attributes
)]
#![warn(clippy::nursery, clippy::pedantic)]
#![allow(clippy::module_name_repetitions, clippy::module_inception)]

mod array_map;
mod array_map_ext;
mod entry;
mod errors;
pub mod ext;
mod external_trait_impls;
pub mod iter;
mod macros;
mod occupied;
mod utils;
mod vacant;

pub use crate::array_map::*;
pub use array_map_ext::*;
pub use entry::*;
pub use errors::*;
pub use occupied::*;
pub use vacant::*;
