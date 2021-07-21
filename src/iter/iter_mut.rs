use core::{fmt, slice};

#[must_use]
pub struct IterMut<'a, K, V>(slice::IterMut<'a, Option<(K, V)>>);

impl<'a, K, V> IterMut<'a, K, V> {
    pub fn new(entries: &'a mut [Option<(K, V)>]) -> Self {
        Self(entries.iter_mut())
    }

    pub(crate) fn iter(&self) -> slice::Iter<'_, Option<(K, V)>> {
        self.0.as_slice().iter()
    }
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        for entry in &mut self.0 {
            if let Some((k, v)) = entry {
                return Some((k, v));
            }
        }

        None
    }
}

impl<'a, K: fmt::Debug, V: fmt::Debug> fmt::Debug for IterMut<'a, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.0.as_slice().iter()).finish()
    }
}
