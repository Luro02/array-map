use core::fmt;
use core::iter::FusedIterator;

use crate::raw::RawTableIter;

use super::Iter;

/// An iterator over the immutable values of an `ArrayMap`.
///
/// This struct is created by [`ArrayMap::values`]. See its documentation for
/// more.
///
/// [`ArrayMap::values`]: crate::ArrayMap::values
#[must_use]
pub struct Values<'a, K, V, R: RawTableIter<(K, V)>>(Iter<'a, K, V, R>);

impl<'a, K, V, R: RawTableIter<(K, V)>> Values<'a, K, V, R> {
    pub(crate) fn new(iter: Iter<'a, K, V, R>) -> Self {
        Self(iter)
    }
}

impl<'a, K, V, R: RawTableIter<(K, V)>> Iterator for Values<'a, K, V, R> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, v)| v)
    }
}

impl<'a, K: fmt::Debug, V: fmt::Debug, R: RawTableIter<(K, V)>> fmt::Debug for Values<'a, K, V, R>
where
    Iter<'a, K, V, R>: Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries((*self).clone()).finish()
    }
}

impl<'a, K, V, R: RawTableIter<(K, V)>> Clone for Values<'a, K, V, R>
where
    Iter<'a, K, V, R>: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<'a, K, V, R> FusedIterator for Values<'a, K, V, R>
where
    Iter<'a, K, V, R>: FusedIterator,
    R: RawTableIter<(K, V)>,
{
}
