use core::{fmt, slice};
use core::iter::FusedIterator;

#[must_use]
pub struct IterMut<'a, K, V>(slice::IterMut<'a, Option<(K, V)>>);

impl<'a, K, V> IterMut<'a, K, V> {
    pub(crate) fn new(entries: &'a mut [Option<(K, V)>]) -> Self {
        Self(entries.iter_mut())
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &(K, V)> {
        self.0.as_slice().iter().flatten()
    }

    #[inline]
    #[must_use]
    fn len(&self) -> usize {
        self.iter().count()
    }
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        let (k, v) = (&mut self.0).flatten().next()?;
        Some((k, v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<'a, K: fmt::Debug, V: fmt::Debug> fmt::Debug for IterMut<'a, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<'a, K, V> FusedIterator for IterMut<'a, K, V> {}

impl<'a, K, V> ExactSizeIterator for IterMut<'a, K, V> {}
