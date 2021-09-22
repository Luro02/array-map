use core::iter::FusedIterator;

use crate::invariant;

// start index + total length with wrapping?
// this should make it possible to skip parts?

#[must_use]
pub(crate) struct IterCircular<'a, T> {
    slice: &'a [T],
    index: usize,
    stop: usize,
    exhausted: bool,
}

impl<'a, T> IterCircular<'a, T> {
    pub(crate) fn new(start: usize, slice: &'a [T]) -> Self {
        assert!(start < slice.len());

        Self {
            slice,
            index: start,
            stop: start,
            exhausted: false,
        }
    }

    #[inline]
    #[must_use]
    const fn distance_to(&self, destination: usize) -> usize {
        if self.index > destination {
            (self.slice.len() - self.index) + destination
        } else {
            destination - self.index
        }
    }

    #[inline]
    #[must_use]
    const fn decrement_index_wrapping(&self, index: usize) -> usize {
        if index == 0 {
            self.slice.len() - 1
        } else {
            index - 1
        }
    }

    #[inline]
    #[must_use]
    const fn increment_index_wrapping(&self, index: usize) -> usize {
        if index + 1 == self.slice.len() {
            0
        } else {
            index + 1
        }
    }

    #[inline]
    #[must_use]
    const fn is_exhausted(&self) -> bool {
        self.index == self.stop && self.exhausted
    }
}

impl<'a, T> Iterator for IterCircular<'a, T> {
    type Item = (usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_exhausted() {
            return None;
        }

        let index = self.index;
        invariant!(self.index < self.slice.len());
        self.index = self.increment_index_wrapping(index);

        if self.index == self.stop {
            self.exhausted = true;
        }

        Some((index, &self.slice[index]))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let distance = self.distance_to(self.stop);
        (distance, Some(distance))
    }
}

impl<'a, T> DoubleEndedIterator for IterCircular<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.is_exhausted() {
            return None;
        }

        self.stop = self.decrement_index_wrapping(self.stop);
        if self.index == self.stop {
            self.exhausted = true;
        }
        Some((self.stop, &self.slice[self.stop]))
    }
}

impl<'a, T> FusedIterator for IterCircular<'a, T> {}

impl<'a, T> ExactSizeIterator for IterCircular<'a, T> {}

#[cfg(test)]
mod tests {
    use crate::ext::IteratorExt;

    use super::*;

    use pretty_assertions::assert_eq;

    fn numbered_array<const N: usize>() -> [usize; N] {
        let mut index = 0;

        [(); N].map(|_| {
            index += 1;
            index - 1
        })
    }

    #[test]
    fn test_circular_fixed() {
        assert_eq!(
            IterCircular::new(0, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]).try_collect::<[Option<_>; 10]>(),
            Ok([
                Some((0, &0)),
                Some((1, &1)),
                Some((2, &2)),
                Some((3, &3)),
                Some((4, &4)),
                Some((5, &5)),
                Some((6, &6)),
                Some((7, &7)),
                Some((8, &8)),
                Some((9, &9)),
            ])
        );

        assert_eq!(
            IterCircular::new(4, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]).try_collect::<[Option<_>; 10]>(),
            Ok([
                Some((4, &4)),
                Some((5, &5)),
                Some((6, &6)),
                Some((7, &7)),
                Some((8, &8)),
                Some((9, &9)),
                Some((0, &0)),
                Some((1, &1)),
                Some((2, &2)),
                Some((3, &3)),
            ])
        );
    }

    #[test]
    fn test_circular_fixed_reverse() {
        assert_eq!(
            IterCircular::new(0, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9])
                .rev()
                .try_collect::<[Option<_>; 10]>(),
            Ok([
                Some((9, &9)),
                Some((8, &8)),
                Some((7, &7)),
                Some((6, &6)),
                Some((5, &5)),
                Some((4, &4)),
                Some((3, &3)),
                Some((2, &2)),
                Some((1, &1)),
                Some((0, &0)),
            ])
        );

        assert_eq!(
            IterCircular::new(4, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9])
                .rev()
                .try_collect::<[Option<_>; 10]>(),
            Ok([
                Some((3, &3)),
                Some((2, &2)),
                Some((1, &1)),
                Some((0, &0)),
                Some((9, &9)),
                Some((8, &8)),
                Some((7, &7)),
                Some((6, &6)),
                Some((5, &5)),
                Some((4, &4)),
            ])
        );
    }

    #[test]
    fn test_circular_arbitrary() {
        let array: [_; 100] = numbered_array();

        for i in 0..array.len() {
            for (idx, value) in IterCircular::new(i, &array) {
                assert_eq!(&idx, value);
                assert_eq!(&array[idx], value);
            }
        }
    }

    #[test]
    fn test_circular_arbitrary_reverse() {
        let array: [_; 100] = numbered_array();

        for i in 0..array.len() {
            let values: [_; 100] = IterCircular::new(i, &array).try_collect().unwrap();
            let mut rvalues_expected = values.clone();
            rvalues_expected.reverse();

            let rvalues: [_; 100] = IterCircular::new(i, &array).rev().try_collect().unwrap();

            assert_eq!(rvalues_expected, rvalues);

            for (idx, value) in values {
                assert_eq!(&idx, value);
                assert_eq!(&array[idx], value);
            }
        }
    }
}
