use core::iter;
use core::ops::Range;
use core::slice;

#[must_use]
pub struct IterCircular<'a, T> {
    iter: iter::Chain<
        iter::Zip<Range<usize>, slice::Iter<'a, T>>,
        iter::Zip<Range<usize>, slice::Iter<'a, T>>,
    >,
}

impl<'a, T> IterCircular<'a, T> {
    pub(crate) fn new(start: usize, slice: &'a [T]) -> Self {
        assert!(start < slice.len());

        Self {
            iter: (start..slice.len())
                .zip(slice[start..].iter())
                .chain((0..start).zip(slice[..start].iter())),
        }
    }
}

impl<'a, T> Iterator for IterCircular<'a, T> {
    type Item = (usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    fn array_collect<T, I: IntoIterator<Item = T>, const N: usize>(iter: I) -> [Option<T>; N] {
        let mut iter = iter.into_iter();

        [(); N].map(|_| iter.next())
    }

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
            array_collect::<_, _, 10>(IterCircular::new(0, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9])),
            [
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
            ]
        );

        assert_eq!(
            array_collect::<_, _, 10>(IterCircular::new(4, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9])),
            [
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
            ]
        );
    }

    #[test]
    fn test_circular_abitrary() {
        let array: [_; 100] = numbered_array();

        for i in 0..array.len() {
            for (idx, value) in IterCircular::new(i, &array) {
                assert_eq!(&idx, value);
                assert_eq!(&array[idx], value);
            }
        }
    }
}
