use core::{array, slice};

pub trait ToIter<T> {
    type Iter<'b>: Iterator<Item = &'b T>
    where
        T: 'b,
        Self: 'b;

    /// Returns an immutable iterator over the remaining items in the mutable
    /// iterator.
    fn iter(&self) -> Self::Iter<'_>;
}

impl<'a, T> ToIter<T> for slice::IterMut<'a, T> {
    type Iter<'b>
    where
        T: 'b,
        Self: 'b,
    = slice::Iter<'b, T>;

    fn iter(&self) -> Self::Iter<'_> {
        self.as_slice().iter()
    }
}

impl<T, const N: usize> ToIter<T> for array::IntoIter<T, N> {
    type Iter<'b>
    where
        T: 'b,
    = slice::Iter<'b, T>;

    fn iter(&self) -> Self::Iter<'_> {
        self.as_slice().iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ext::IteratorExt;

    #[test]
    fn test_slice_mut() {
        let mut array = [1, 2, 3];
        let slice: &mut [i32] = &mut array[1..];
        let mut iter_mut = slice.iter_mut();

        assert_eq!(
            iter_mut.iter().try_collect::<[Option<_>; 3]>().unwrap(),
            [Some(&2), Some(&3), None]
        );
        assert_eq!(iter_mut.next(), Some(&mut 2));

        assert_eq!(
            iter_mut.iter().try_collect::<[Option<_>; 3]>().unwrap(),
            [Some(&3), None, None]
        );
        assert_eq!(iter_mut.next(), Some(&mut 3));

        assert_eq!(
            iter_mut.iter().try_collect::<[Option<_>; 3]>().unwrap(),
            [None, None, None]
        );
        assert_eq!(iter_mut.next(), None);
    }

    #[test]
    fn test_array() {
        let mut iter = array::IntoIter::new([1, 2, 3]);

        assert_eq!(
            iter.iter().try_collect::<[Option<_>; 4]>().unwrap(),
            [Some(&1), Some(&2), Some(&3), None]
        );
        assert_eq!(iter.next(), Some(1));

        assert_eq!(
            iter.iter().try_collect::<[Option<_>; 4]>().unwrap(),
            [Some(&2), Some(&3), None, None]
        );
        assert_eq!(iter.next(), Some(2));

        assert_eq!(
            iter.iter().try_collect::<[Option<_>; 4]>().unwrap(),
            [Some(&3), None, None, None]
        );
        assert_eq!(iter.next(), Some(3));

        assert_eq!(
            iter.iter().try_collect::<[Option<_>; 4]>().unwrap(),
            [None, None, None, None]
        );
        assert_eq!(iter.next(), None);
    }
}
