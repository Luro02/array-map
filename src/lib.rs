#![cfg_attr(not(feature = "std"), no_std)]
#![feature(
    array_map,
    const_generics,
    array_methods,
    never_type,
    maybe_uninit_uninit_array,
    maybe_uninit_array_assume_init
)]
#![warn(clippy::nursery, clippy::pedantic)]
#![allow(clippy::module_name_repetitions, clippy::module_inception)]

mod array_map;
mod entry;
mod errors;
mod external_trait_impls;
pub mod iter;
mod macros;
mod occupied;
mod utils;
mod vacant;

pub use crate::array_map::*;
pub use entry::*;
pub use errors::*;
pub use occupied::*;
pub use vacant::*;
