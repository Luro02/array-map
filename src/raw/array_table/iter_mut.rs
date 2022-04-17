use core::{iter, slice};

use crate::ext::ToIter;

pub struct IterMut<'a, T> {
    iter: slice::IterMut<'a, Option<T>>,
}

impl<'a, T> IterMut<'a, T> {
    pub(crate) fn new(iter: slice::IterMut<'a, Option<T>>) -> Self {
        Self { iter }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.by_ref().flatten().next()
    }
}

impl<'a, T> ToIter for IterMut<'a, T> {
    type Item = T;
    type Iter<'b> = iter::Flatten<<slice::IterMut<'b, Option<T>> as ToIter>::Iter<'b>>
    where
        Self::Item: 'b,
        Self: 'b;

    fn iter(&self) -> Self::Iter<'_> {
        self.iter.iter().flatten()
    }
}
