use core::fmt;
use core::iter::FusedIterator;

use crate::ext::ToIter;
use crate::raw::RawTableIter;

#[must_use]
pub struct IterMut<'a, K: 'a, V: 'a, R: RawTableIter<(K, V)>> {
    iter: <R as RawTableIter<(K, V)>>::IterMut<'a>,
}

impl<'a, K: 'a, V: 'a, R: RawTableIter<(K, V)>> IterMut<'a, K, V, R> {
    pub(crate) fn new(table: &'a mut R) -> Self {
        Self {
            iter: <R as RawTableIter<(K, V)>>::iter_mut(table),
        }
    }
}

impl<'a, K: 'a, V: 'a, R: RawTableIter<(K, V)>> Iterator for IterMut<'a, K, V, R> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some((key, value)) => Some((key, value)),
            None => None,
        }
    }
}

impl<'a, K: 'a, V: 'a, R: RawTableIter<(K, V)>> fmt::Debug for IterMut<'a, K, V, R>
where
    K: fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter.iter()).finish()
    }
}

impl<'a, K: 'a, V: 'a, R> ToIter<(K, V)> for IterMut<'a, K, V, R>
where
    R: RawTableIter<(K, V)>,
{
    type Iter<'b>
    where
        K: 'b,
        V: 'b,
    = <R::IterMut<'a> as ToIter<(K, V)>>::Iter<'b>;

    fn iter(&self) -> Self::Iter<'_> {
        self.iter.iter()
    }
}

impl<'a, K: 'a, V: 'a, R: RawTableIter<(K, V)>> FusedIterator for IterMut<'a, K, V, R>
//
// where
//     <R as RawTableIter<(K, V)>>::IterMut<'a>: FusedIterator
{
}
