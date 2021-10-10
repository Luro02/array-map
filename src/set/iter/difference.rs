use core::hash::Hash;
use core::iter::FusedIterator;

use crate::set::{Set, SetIter};

/// A lazy iterator producing elements in the difference of [`Set`]s.
///
/// This `struct` is created by [`Set::difference`].
pub struct Difference<'a, T: 'a, A: SetIter<T>, B: Set<T>> {
    a: A::Iter<'a>,
    b: &'a B,
}

impl<'a, T, A: SetIter<T>, B: Set<T>> Difference<'a, T, A, B> {
    pub(crate) fn new(a: &'a A, b: &'a B) -> Self {
        Self { a: a.iter(), b }
    }
}

impl<'a, T: Hash + Eq, A: SetIter<T>, B: Set<T>> Iterator for Difference<'a, T, A, B> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        for element in self.a.by_ref() {
            if !self.b.contains(element) {
                return Some(element);
            }
        }

        None
    }
}

impl<'a, T: Hash + Eq, A: SetIter<T>, B: Set<T>> DoubleEndedIterator for Difference<'a, T, A, B>
where
    A::Iter<'a>: DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        while let Some(element) = self.a.next_back() {
            if !self.b.contains(element) {
                return Some(element);
            }
        }

        None
    }
}

impl<'a, T, A, B> FusedIterator for Difference<'a, T, A, B>
where
    A::Iter<'a>: FusedIterator,
    T: Hash + Eq,
    A: SetIter<T>,
    B: Set<T>,
{
}

impl<'a, T, A: SetIter<T>, B: Set<T>> Clone for Difference<'a, T, A, B>
where
    A::Iter<'a>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            a: self.a.clone(),
            b: self.b,
        }
    }
}
