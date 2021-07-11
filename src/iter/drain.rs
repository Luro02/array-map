use core::hash::{BuildHasher, Hash};

use super::DrainFilter;
use crate::{ArrayMap, DefaultHashBuilder};

pub struct Drain<'a, K, V, const N: usize, B: BuildHasher = DefaultHashBuilder>
where
    K: Hash + Eq,
{
    inner: DrainFilter<'a, K, V, fn(&K, &mut V) -> bool, N, B>,
}

impl<'a, K, V, B: BuildHasher, const N: usize> Drain<'a, K, V, N, B>
where
    K: Hash + Eq,
{
    pub fn new(map: &'a mut ArrayMap<K, V, N, B>) -> Self {
        Self {
            inner: DrainFilter::new(|_, _| true, map),
        }
    }
}

impl<'a, K, V, B: BuildHasher, const N: usize> Iterator for Drain<'a, K, V, N, B>
where
    K: Eq + Hash,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::{arraymap, ArrayMap};

    fn array_collect<T, I: IntoIterator<Item = T>, const N: usize>(iter: I) -> [Option<T>; N] {
        let mut iter = iter.into_iter();

        [(); N].map(|_| iter.next())
    }

    #[test]
    fn test_drain() {
        let mut map = arraymap! {
            0 => "a",
            1 => "b",
            2 => "c",
            3 => "d",
        };

        let mut drained: [_; 4] = array_collect(map.drain());
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
        let mut map = arraymap! {
            0 => "a",
            1 => "b",
            2 => "c",
            3 => "d",
        };

        map.drain();
        assert_eq!(map, ArrayMap::new());
    }
}
