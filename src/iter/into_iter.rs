use core::array;
use core::fmt::{self, Debug};
use core::iter::FusedIterator;

#[must_use]
pub struct IntoIter<K, V, const N: usize>(array::IntoIter<Option<(K, V)>, N>);

impl<K, V, const N: usize> IntoIter<K, V, N> {
    pub(crate) fn new(entries: [Option<(K, V)>; N]) -> Self {
        Self(entries.into_iter())
    }

    fn iter(&self) -> impl Iterator<Item = &(K, V)> {
        self.0.as_slice().iter().flat_map(Option::as_ref)
    }

    fn len(&self) -> usize {
        self.iter().count()
    }
}

impl<K, V, const N: usize> Iterator for IntoIter<K, V, N> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        (&mut self.0).flatten().next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<K, V, const N: usize> fmt::Debug for IntoIter<K, V, N>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<K, V, const N: usize> ExactSizeIterator for IntoIter<K, V, N> {}

impl<K, V, const N: usize> FusedIterator for IntoIter<K, V, N> {}
