use core::iter;
use core::ops::Range;
use core::slice;

// TODO: test this!
#[must_use]
pub struct IterCircular<'a, T> {
    iter: iter::Chain<
        iter::Zip<Range<usize>, slice::Iter<'a, T>>,
        iter::Zip<Range<usize>, slice::Iter<'a, T>>,
    >,
}

impl<'a, T> IterCircular<'a, T> {
    pub fn new(start: usize, slice: &'a [T]) -> Self {
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
