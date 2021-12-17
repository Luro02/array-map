use core::fmt;
use core::iter::FusedIterator;
use core::marker::PhantomData;

use crate::raw::RawTableIter;

#[must_use]
pub struct Iter<'a, K, V, R: 'a + RawTableIter<(K, V)>>(R::Iter<'a>, PhantomData<&'a (K, V)>);

impl<'a, K, V, R: RawTableIter<(K, V)>> Iter<'a, K, V, R> {
    pub(crate) fn new(table: &'a R) -> Self {
        Self(<R as RawTableIter<_>>::iter(table), PhantomData)
    }
}

impl<'a, K: 'a, V: 'a, R: RawTableIter<(K, V)>> Iterator for Iter<'a, K, V, R> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next() {
            Some((key, value)) => Some((key, value)),
            None => None,
        }
    }
}

impl<'a, K, V, R: RawTableIter<(K, V)>> Clone for Iter<'a, K, V, R>
where
    R::Iter<'a>: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<'a, K: fmt::Debug, V: fmt::Debug, R: RawTableIter<(K, V)>> fmt::Debug for Iter<'a, K, V, R>
where
    R::Iter<'a>: Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, K, V, R: RawTableIter<(K, V)>> FusedIterator for Iter<'a, K, V, R>
//
where
    R::Iter<'a>: FusedIterator
{
}
