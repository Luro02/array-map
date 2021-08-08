use core::fmt;
use core::iter::FusedIterator;

use crate::ext::IntoImmutableIter;
use crate::raw::RawTableIter;

use super::IterMut;

/// An iterator over the mutable values of an `ArrayMap`.
///
/// This struct is created by [`ArrayMap::values_mut`]. See its documentation
/// for more.
///
/// [`ArrayMap::values_mut`]: crate::ArrayMap::values_mut
#[must_use]
pub struct ValuesMut<'a, K: 'a, V: 'a, R: RawTableIter<(K, V)>>(IterMut<'a, K, V, R>);

impl<'a, K: 'a, V: 'a, R: RawTableIter<(K, V)>> ValuesMut<'a, K, V, R> {
    pub(crate) fn new(iter: IterMut<'a, K, V, R>) -> Self {
        Self(iter)
    }
}

impl<'a, K: 'a, V: 'a, R: RawTableIter<(K, V)>> Iterator for ValuesMut<'a, K, V, R> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, v)| v)
    }
}

impl<'a, K: 'a, V: 'a, R: RawTableIter<(K, V)>> fmt::Debug for ValuesMut<'a, K, V, R>
where
    K: fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.0.iter()).finish()
    }
}

impl<'a, K, V, R: RawTableIter<(K, V)>> FusedIterator for ValuesMut<'a, K, V, R>
//
where
    IterMut<'a, K, V, R>: FusedIterator
{
}
