use core::hash::{BuildHasher, Hash};

use super::DrainFilter;
use crate::{ArrayMap, DefaultHashBuilder};

pub struct Drain<'a, K, V, const N: usize, B: BuildHasher = DefaultHashBuilder>
where
    K: Hash + Eq,
{
    inner: DrainFilter<'a, K, V, fn(&K, &mut V) -> bool, N, B>,
}

impl<'a, K, V, B: BuildHasher, const N: usize> Drain<'a, K, V, N, B>
where
    K: Hash + Eq,
{
    pub fn new(map: &'a mut ArrayMap<K, V, N, B>) -> Self {
        Self {
            inner: DrainFilter::new(|_, _| true, map),
        }
    }
}

impl<'a, K, V, B: BuildHasher, const N: usize> Iterator for Drain<'a, K, V, N, B>
where
    K: Eq + Hash,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<'a, K, V, B, const N: usize> Drop for Drain<'a, K, V, N, B>
where
    B: BuildHasher,
    K: Eq + Hash,
{
    fn drop(&mut self) {
        for _ in self {}
    }
}
