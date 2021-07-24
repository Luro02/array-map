use core::{slice, fmt};
use core::iter::{self, FusedIterator};

#[must_use]
pub struct Iter<'a, K, V>(iter::Flatten<slice::Iter<'a, Option<(K, V)>>>);

impl<'a, K, V> Iter<'a, K, V> {
    pub(crate) fn new(entries: &'a [Option<(K, V)>]) -> Self {
        Self(entries.iter().flatten())
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, v)| (k, v))
    }
}

impl<'a, K, V> Clone for Iter<'a, K, V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<'a, K: fmt::Debug, V: fmt::Debug> fmt::Debug for Iter<'a, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, K, V> FusedIterator for Iter<'a, K, V> {}
