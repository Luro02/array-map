use core::hash::{BuildHasher, Hash};
use core::mem;

use crate::{ArrayMap, DefaultHashBuilder};

pub struct DrainFilter<'a, K, V, F, const N: usize, B: BuildHasher = DefaultHashBuilder>
where
    F: FnMut(&K, &mut V) -> bool,
    K: Hash + Eq,
{
    f: F,
    map: &'a mut ArrayMap<K, V, N, B>,
    index: usize,
}

impl<'a, K, V, F, B: BuildHasher, const N: usize> DrainFilter<'a, K, V, F, N, B>
where
    F: FnMut(&K, &mut V) -> bool,
    K: Hash + Eq,
{
    pub fn new(f: F, map: &'a mut ArrayMap<K, V, N, B>) -> Self {
        Self { f, map, index: 0 }
    }
}

impl<'a, K, V, F, B: BuildHasher, const N: usize> Iterator for DrainFilter<'a, K, V, F, N, B>
where
    F: FnMut(&K, &mut V) -> bool,
    K: Eq + Hash,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < N {
            let index = self.index;
            self.index += 1;

            if let Some((key, value)) = self.map.get_key_value_mut_index(index) {
                if (self.f)(key, value) {
                    let result = self.map.remove_entry_index(index);

                    // removing an entry from the map, may cause other entries to fill in the empty spot
                    self.index = index;

                    return result;
                }
            }
        }

        None
    }
}

impl<'a, K, V, F, B, const N: usize> Drop for DrainFilter<'a, K, V, F, N, B>
where
    B: BuildHasher,
    F: FnMut(&K, &mut V) -> bool,
    K: Eq + Hash,
{
    fn drop(&mut self) {
        self.for_each(mem::drop)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::array_map;

    fn array_collect<T, I: IntoIterator<Item = T>, const N: usize>(iter: I) -> [Option<T>; N] {
        let mut iter = iter.into_iter();

        [(); N].map(|_| iter.next())
    }

    #[test]
    fn test_drain_filter_drop() {
        let mut map = array_map! {
            0 => "a",
            1 => "b",
            2 => "c",
            3 => "d",
        };

        map.drain_filter(|_, v| {
            *v = "l";
            false
        });

        assert_eq!(
            map,
            array_map! {
                0 => "l",
                1 => "l",
                2 => "l",
                3 => "l",
            }
        );
    }

    #[test]
    fn test_drain_filter() {
        let mut map = array_map! {
            0 => "a",
            1 => "b",
            2 => "c",
            3 => "d",
        };

        let drain = map.drain_filter(|k, v| {
            if k % 2 == 0 {
                true
            } else {
                *v = "u";
                false
            }
        });

        let mut dropped: [_; 2] = array_collect(drain);
        dropped.sort_unstable();

        assert_eq!(dropped, [Some((0, "a")), Some((2, "c"))]);

        assert_eq!(
            map,
            array_map! {
                @infer,
                1 => "u",
                3 => "u",
            }
            .unwrap()
        );
    }
}
