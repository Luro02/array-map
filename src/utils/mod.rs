mod iter_circular;
mod iter_entries;
mod try_extend;
mod try_from_iterator;

pub use iter_circular::*;
pub use iter_entries::*;
pub use try_extend::*;
pub use try_from_iterator::*;

use core::borrow::Borrow;
use core::hash::{BuildHasher, Hash, Hasher};

#[must_use]
pub(crate) fn make_hash<K, Q, B>(hash_builder: &B, value: &Q) -> u64
where
    K: Borrow<Q>,
    Q: Hash + ?Sized,
    B: BuildHasher,
{
    let mut hasher = hash_builder.build_hasher();
    value.hash(&mut hasher);
    hasher.finish()
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
