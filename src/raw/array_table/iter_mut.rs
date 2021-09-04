use core::{iter, slice};

use crate::ext::ToIter;

pub struct IterMut<'a, T> {
    iter: slice::IterMut<'a, Option<T>>,
}

impl<'a, T> IterMut<'a, T> {
    pub(super) fn new(iter: slice::IterMut<'a, Option<T>>) -> Self {
        Self { iter }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.by_ref().flatten().next()
    }
}

impl<'a, T> ToIter<T> for IterMut<'a, T> {
    type Iter<'b>
    where
        T: 'b,
    = iter::Flatten<<slice::IterMut<'b, Option<T>> as ToIter<Option<T>>>::Iter<'b>>;

    fn iter(&self) -> Self::Iter<'_> {
        self.iter.iter().flatten()
    }
}
