use core::hash::{BuildHasher, Hash};

use super::DrainFilter;
use crate::ArrayMap;

/// A draining iterator over entries of an `ArrayMap`.
///
/// This struct is created by [`ArrayMap::drain`]. See its documentation for more.
///
/// [`ArrayMap::drain`]: crate::ArrayMap::drain
pub struct Drain<'a, K, V, B: BuildHasher, const N: usize>
where
    K: Hash + Eq,
{
    inner: DrainFilter<'a, K, V, fn(&K, &mut V) -> bool, B, N>,
}

impl<'a, K, V, B: BuildHasher, const N: usize> Drain<'a, K, V, B, N>
where
    K: Hash + Eq,
{
    pub(crate) fn new(map: &'a mut ArrayMap<K, V, N, B>) -> Self {
        Self {
            inner: DrainFilter::new(|_, _| true, map),
        }
    }
}

impl<'a, K, V, B: BuildHasher, const N: usize> Iterator for Drain<'a, K, V, B, N>
where
    K: Eq + Hash,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

#[cfg(all(test, feature = "ahash"))]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::ext::IteratorExt;
    use crate::{array_map, ArrayMap};

    #[test]
    fn test_drain() {
        let mut map = array_map! {
            0 => "a",
            1 => "b",
            2 => "c",
            3 => "d",
        };

        let mut drained: [Option<(i32, &str)>; 4] = map.drain().try_collect().unwrap();
        drained.sort_unstable();

        assert_eq!(
            drained,
            [
                Some((0, "a")),
                Some((1, "b")),
                Some((2, "c")),
                Some((3, "d"))
            ]
        );
        assert_eq!(map, ArrayMap::new());
    }

    #[test]
    fn test_drain_drop() {
        let mut map = array_map! {
            0 => "a",
            1 => "b",
            2 => "c",
            3 => "d",
        };

        map.drain();
        assert_eq!(map, ArrayMap::new());
    }
}
