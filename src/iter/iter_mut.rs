use core::iter;
use core::slice;

#[must_use]
pub struct IterMut<'a, K, V>(iter::Flatten<slice::IterMut<'a, Option<(K, V)>>>);

impl<'a, K, V> IterMut<'a, K, V> {
    pub fn new(entries: &'a mut [Option<(K, V)>]) -> Self {
        Self(entries.iter_mut().flatten())
    }
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((k, v)) = self.0.next() {
            Some((k, v))
        } else {
            None
        }
    }
}
