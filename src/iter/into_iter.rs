use core::array;
use core::iter;

#[must_use]
pub struct IntoIter<K, V, const N: usize>(iter::Flatten<array::IntoIter<Option<(K, V)>, N>>);

impl<K, V, const N: usize> IntoIter<K, V, N> {
    pub fn new(entries: [Option<(K, V)>; N]) -> Self {
        Self(array::IntoIter::new(entries).flatten())
    }
}

impl<K, V, const N: usize> Iterator for IntoIter<K, V, N> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
