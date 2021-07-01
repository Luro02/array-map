use core::iter;
use core::slice;

#[must_use]
pub struct Iter<'a, K, V>(iter::Flatten<slice::Iter<'a, Option<(K, V)>>>);

impl<'a, K, V> Iter<'a, K, V> {
    pub fn new(entries: &'a [Option<(K, V)>]) -> Self {
        Self(entries.iter().flatten())
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, v)| (k, v))
    }
}
