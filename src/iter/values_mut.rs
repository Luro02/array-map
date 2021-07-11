use core::fmt;

use super::IterMut;

/// An iterator over the mutable values of an `ArrayMap`.
///
/// This struct is created by [`ArrayMap::values_mut`]. See its documentation for more.
///
/// [`ArrayMap::values_mut`]: crate::ArrayMap::values_mut
#[must_use]
pub struct ValuesMut<'a, K, V>(IterMut<'a, K, V>);

impl<'a, K, V> ValuesMut<'a, K, V> {
    pub(crate) fn new(iter: IterMut<'a, K, V>) -> Self {
        Self(iter)
    }
}

impl<'a, K, V> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, v)| v)
    }
}

impl<'a, K: fmt::Debug, V: fmt::Debug> fmt::Debug for ValuesMut<'a, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.0.iter().flatten().map(|(_, v)| v))
            .finish()
    }
}
