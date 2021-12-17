use core::hash::Hash;
use core::iter::FusedIterator;

use crate::set::{Set, SetIter};

/// A lazy iterator producing elements in the intersection of [`Set`]s.
///
/// This `struct` is created by [`Set::intersection`].
pub struct Intersection<'a, T: 'a, A: 'a + SetIter<T>, B: Set<T>> {
    a: A::Iter<'a>,
    b: &'a B,
}

impl<'a, T, A: SetIter<T>, B: Set<T>> Intersection<'a, T, A, B> {
    pub(crate) fn new(a: &'a A, b: &'a B) -> Self {
        Self { a: a.iter(), b }
    }
}

impl<'a, T: Hash + Eq, A: SetIter<T>, B: Set<T>> Iterator for Intersection<'a, T, A, B> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        for element in self.a.by_ref() {
            if self.b.contains(element) {
                return Some(element);
            }
        }

        None
    }
}

impl<'a, T: Hash + Eq, A, B: Set<T>> DoubleEndedIterator for Intersection<'a, T, A, B>
where
    A: 'a + SetIter<T>,
    A::Iter<'a>: DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        while let Some(element) = self.a.next_back() {
            if self.b.contains(element) {
                return Some(element);
            }
        }

        None
    }
}

impl<'a, T: Hash + Eq, A, B: Set<T>> FusedIterator for Intersection<'a, T, A, B>
//
where
    A: 'a + SetIter<T>,
    A::Iter<'a>: FusedIterator,
{
}

impl<'a, T, A, B: Set<T>> Clone for Intersection<'a, T, A, B>
where
    A: 'a + SetIter<T>,
    A::Iter<'a>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            a: self.a.clone(),
            b: self.b,
        }
    }
}
