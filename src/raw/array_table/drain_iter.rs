use core::iter::FusedIterator;
use core::{array, fmt, iter, slice};

use crate::ext::ToIter;

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct DrainIter<T, const N: usize>(array::IntoIter<Option<T>, N>);

impl<T, const N: usize> DrainIter<T, N> {
    pub(crate) fn new(data: [Option<T>; N]) -> Self {
        Self(data.into_iter())
    }
}

impl<T, const N: usize> Iterator for DrainIter<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.by_ref().flatten().next()
    }
}

impl<T, const N: usize> FusedIterator for DrainIter<T, N> {}

impl<T, const N: usize> ToIter<T> for DrainIter<T, N> {
    type Iter<'a>
    where
        T: 'a,
    = iter::Flatten<slice::Iter<'a, Option<T>>>;

    fn iter(&self) -> Self::Iter<'_> {
        self.0.iter().flatten()
    }
}

impl<T: fmt::Debug, const N: usize> fmt::Debug for DrainIter<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}
