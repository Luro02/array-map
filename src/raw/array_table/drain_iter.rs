use core::array;

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct DrainIter<T, const N: usize>(array::IntoIter<Option<T>, N>);

impl<T, const N: usize> DrainIter<T, N> {
    pub(super) fn new(data: [Option<T>; N]) -> Self {
        Self(array::IntoIter::new(data))
    }
}

impl<T, const N: usize> Iterator for DrainIter<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.by_ref().flatten().next()
    }
}
