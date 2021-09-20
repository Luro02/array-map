use core::fmt;
use core::hash::{BuildHasher, Hash};
use core::iter::FusedIterator;
use core::ops::Range;

use crate::IndexMap;

pub struct DrainRange<'a, K: Hash + Eq, V, B: BuildHasher, const N: usize> {
    map: &'a mut IndexMap<K, V, N, B>,
    remaining: usize,
    index: usize,
}

impl<'a, K, V, B, const N: usize> DrainRange<'a, K, V, B, N>
where
    K: Hash + Eq,
    B: BuildHasher,
{
    #[must_use]
    pub(crate) fn new(map: &'a mut IndexMap<K, V, N, B>, range: Range<usize>) -> Self {
        Self {
            map,
            remaining: range.end - range.start,
            index: range.start,
        }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&'_ K, &'_ V)> {
        (self.index..(self.index + self.remaining))
            .flat_map(|index| self.map.get_index_entry(index))
    }
}

impl<'a, K, V, B, const N: usize> Iterator for DrainRange<'a, K, V, B, N>
where
    K: Hash + Eq,
    B: BuildHasher,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        self.remaining -= 1;

        self.map.shift_remove_index(self.index)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<'a, K, V, B, const N: usize> DoubleEndedIterator for DrainRange<'a, K, V, B, N>
where
    K: Hash + Eq,
    B: BuildHasher,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        self.remaining -= 1;

        self.map.shift_remove_index(self.index + self.remaining)
    }
}

impl<'a, K: Hash + Eq, V, B: BuildHasher, const N: usize> Drop for DrainRange<'a, K, V, B, N> {
    fn drop(&mut self) {
        self.for_each(drop);
    }
}

impl<'a, K, V, B, const N: usize> fmt::Debug for DrainRange<'a, K, V, B, N>
where
    K: Hash + Eq + fmt::Debug,
    V: fmt::Debug,
    B: BuildHasher,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<'a, K, V, B, const N: usize> FusedIterator for DrainRange<'a, K, V, B, N>
where
    K: Hash + Eq,
    B: BuildHasher,
{
}

impl<'a, K, V, B, const N: usize> ExactSizeIterator for DrainRange<'a, K, V, B, N>
where
    K: Hash + Eq,
    B: BuildHasher,
{
}
