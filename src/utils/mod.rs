mod iter_circular;
mod iter_entries;
mod try_from_iterator;

pub use iter_circular::*;
pub use iter_entries::*;
pub use try_from_iterator::*;

use core::hash::{BuildHasher, Hash, Hasher};

#[must_use]
pub fn hash_index<H: BuildHasher, Z: Hash>(value: Z, hasher: &H, capacity: usize) -> usize {
    let mut hasher = hasher.build_hasher();
    value.hash(&mut hasher);

    (hasher.finish() as usize) % capacity
}

pub trait ArrayExt<T, const N: usize> {
    fn enumerate(self) -> [(usize, T); N];
}

impl<T, const N: usize> ArrayExt<T, N> for [T; N] {
    fn enumerate(self) -> [(usize, T); N] {
        let mut index = 0;
        self.map(|value| {
            index += 1;
            (index - 1, value)
        })
    }
}
