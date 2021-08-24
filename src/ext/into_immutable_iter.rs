use core::{array, slice};

pub trait IntoImmutableIter<T> {
    type Iter<'b>: Iterator<Item = &'b T>
    where
        T: 'b;

    /// Returns an immutable iterator over the remaining items in the mutable
    /// iterator.
    fn iter(&self) -> Self::Iter<'_>;
}

impl<'a, T> IntoImmutableIter<T> for slice::IterMut<'a, T> {
    type Iter<'b>
    where
        T: 'b,
    = slice::Iter<'b, T>;

    fn iter(&self) -> Self::Iter<'_> {
        self.as_slice().iter()
    }
}

impl<T, const N: usize> IntoImmutableIter<T> for array::IntoIter<T, N> {
    type Iter<'b>
    where
        T: 'b,
    = slice::Iter<'b, T>;

    fn iter(&self) -> Self::Iter<'_> {
        self.as_slice().iter()
    }
}
