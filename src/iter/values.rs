use core::fmt;

use super::Iter;

/// An iterator over the immutable values of an `ArrayMap`.
///
/// This struct is created by [`ArrayMap::values`]. See its documentation for more.
///
/// [`ArrayMap::values`]: crate::ArrayMap::values
#[must_use]
pub struct Values<'a, K, V>(Iter<'a, K, V>);

impl<'a, K, V> Values<'a, K, V> {
    pub(crate) fn new(iter: Iter<'a, K, V>) -> Self {
        Self(iter)
    }
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, v)| v)
    }
}

impl<'a, K: fmt::Debug, V: fmt::Debug> fmt::Debug for Values<'a, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries((*self).clone()).finish()
    }
}

impl<'a, K, V> Clone for Values<'a, K, V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
