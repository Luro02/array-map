use core::fmt;

use super::Iter;

/// An iterator over the keys of an `ArrayMap`.
///
/// This struct is created by [`ArrayMap::keys`]. See its documentation for more.
///
/// [`ArrayMap::keys`]: crate::ArrayMap::keys
#[must_use]
pub struct Keys<'a, K, V>(Iter<'a, K, V>);

impl<'a, K, V> Keys<'a, K, V> {
    pub(crate) fn new(iter: Iter<'a, K, V>) -> Self {
        Self(iter)
    }
}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, _)| k)
    }
}

impl<'a, K: fmt::Debug, V: fmt::Debug> fmt::Debug for Keys<'a, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries((*self).clone()).finish()
    }
}

impl<'a, K, V> Clone for Keys<'a, K, V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
