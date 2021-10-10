use core::hash::Hash;
use core::iter;
use core::iter::FusedIterator;

use crate::set::iter::Difference;
use crate::set::SetIter;

/// A lazy iterator producing elements that are in `A` and not in `B` and then
/// those that are in `B` and not in `A`.
///
/// This `struct` is created by [`Set::symmetric_difference`].
pub struct SymmetricDifference<'a, T: 'a, A: SetIter<T>, B: SetIter<T>> {
    iter: iter::Chain<Difference<'a, T, A, B>, Difference<'a, T, B, A>>,
}

impl<'a, T: Hash + Eq, A: SetIter<T>, B: SetIter<T>> SymmetricDifference<'a, T, A, B> {
    pub(crate) fn new(a: &'a A, b: &'a B) -> Self {
        Self {
            iter: a.difference(b).chain(b.difference(a)),
        }
    }
}

impl<'a, T: Hash + Eq, A: SetIter<T>, B: SetIter<T>> Iterator for SymmetricDifference<'a, T, A, B> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a, T: Hash + Eq, A: SetIter<T>, B: SetIter<T>> DoubleEndedIterator
    for SymmetricDifference<'a, T, A, B>
where
    A::Iter<'a>: DoubleEndedIterator,
    B::Iter<'a>: DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

impl<'a, T: Hash + Eq, A: SetIter<T>, B: SetIter<T>> FusedIterator
    for SymmetricDifference<'a, T, A, B>
where
    A::Iter<'a>: FusedIterator,
{
}

impl<'a, T, A: SetIter<T>, B: SetIter<T>> Clone for SymmetricDifference<'a, T, A, B>
where
    A::Iter<'a>: Clone,
    B::Iter<'a>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}
