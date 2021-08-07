use core::fmt;
use core::iter::FusedIterator;

use crate::raw::RawTableIter;

use super::Iter;

/// An iterator over the keys of an `ArrayMap`.
///
/// This struct is created by [`ArrayMap::keys`]. See its documentation for
/// more.
///
/// [`ArrayMap::keys`]: crate::ArrayMap::keys
#[must_use]
pub struct Keys<'a, K, V, R: RawTableIter<(K, V)>>(Iter<'a, K, V, R>);

impl<'a, K, V, R: RawTableIter<(K, V)>> Keys<'a, K, V, R> {
    pub(crate) fn new(iter: Iter<'a, K, V, R>) -> Self {
        Self(iter)
    }
}

impl<'a, K, V, R: RawTableIter<(K, V)>> Iterator for Keys<'a, K, V, R> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, _)| k)
    }
}

impl<'a, K: fmt::Debug, V: fmt::Debug, R: RawTableIter<(K, V)>> fmt::Debug for Keys<'a, K, V, R>
where
    Iter<'a, K, V, R>: Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries((*self).clone()).finish()
    }
}

impl<'a, K, V, R: RawTableIter<(K, V)>> Clone for Keys<'a, K, V, R>
where
    Iter<'a, K, V, R>: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

// TODO: constrain to Iter?
impl<'a, K, V, R: RawTableIter<(K, V)>> FusedIterator for Keys<'a, K, V, R> {}
