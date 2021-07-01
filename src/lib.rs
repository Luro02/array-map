#![cfg_attr(not(feature = "std"), no_std)]
#![feature(array_map, const_generics)]
#![warn(clippy::nursery, clippy::pedantic)]
#![allow(clippy::module_name_repetitions, clippy::module_inception)]

mod array_map;
mod entry;
pub mod iter;
mod occupied;
mod utils;
mod vacant;

pub use crate::array_map::*;
pub use entry::*;
pub use occupied::*;
pub use vacant::*;
