use core::hash::{BuildHasher, Hash};
use core::mem;

use crate::utils;
use crate::utils::IterCircular;
use crate::DefaultHashBuilder;

#[derive(Debug)]
pub struct OccupiedEntry<'a, K: 'a, V: 'a, const N: usize, H: BuildHasher = DefaultHashBuilder> {
    entries: &'a mut [Option<(K, V)>; N],
    index: usize,
    hasher: &'a H,
    len: &'a mut usize,
}

impl<'a, K, V, const N: usize, H: BuildHasher> OccupiedEntry<'a, K, V, N, H> {
    #[must_use]
    pub(crate) fn new(
        entries: &'a mut [Option<(K, V)>; N],
        index: usize,
        hasher: &'a H,
        len: &'a mut usize,
    ) -> Self {
        debug_assert_eq!(entries.len(), N);
        debug_assert!(index < N);
        debug_assert!(entries[index].is_some());
        debug_assert!(*len > 0);

        Self {
            entries,
            index,
            hasher,
            len,
        }
    }

    #[must_use]
    fn entry(&self) -> (&K, &V) {
        let (key, value) = &self.entries[self.index].as_ref().unwrap();

        (key, value)
    }

    #[must_use]
    pub fn key(&self) -> &K {
        self.entry().0
    }

    #[must_use]
    pub fn get(&self) -> &V {
        self.entry().1
    }

    #[must_use]
    pub fn get_mut(&mut self) -> &mut V {
        &mut self.entries[self.index].as_mut().unwrap().1
    }

    pub fn insert(&mut self, mut value: V) -> V {
        mem::swap(self.get_mut(), &mut value);

        value
    }

    #[must_use]
    pub fn into_mut(self) -> &'a mut V {
        &mut self.entries[self.index].as_mut().unwrap().1
    }
}

#[must_use]
struct IterCollisions<'a, K: 'a, V: 'a, const N: usize, H: BuildHasher = DefaultHashBuilder> {
    iter: IterCircular<'a, Option<(K, V)>>,
    hasher: &'a H,
    start: usize,
}

impl<'a, K: Hash, V, H: BuildHasher, const N: usize> IterCollisions<'a, K, V, N, H> {
    pub fn new(key: &K, entries: &'a [Option<(K, V)>; N], hasher: &'a H) -> Self {
        let start = utils::hash_index(key, hasher, N);

        Self {
            iter: IterCircular::new(start, entries),
            start,
            hasher,
        }
    }
}

// TODO: can be replaced with IterEntries + Map
impl<'a, K: Hash, V, H: BuildHasher, const N: usize> Iterator for IterCollisions<'a, K, V, N, H> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        for (index, entry) in &mut self.iter {
            if let Some((key, _)) = entry {
                // check if the current entry is a collision:
                if utils::hash_index(key, self.hasher, N) == self.start {
                    return Some(index);
                }
            } else {
                // an empty entry has been found, which means that all the following entries keys will have different hashes
                // NOTE: does not increment self.index, because after a None, None will always be returned
                return None;
            }
        }

        None
    }
}

impl<'a, K: Hash + Eq, V, const N: usize, H: BuildHasher> OccupiedEntry<'a, K, V, N, H> {
    fn find_with_hash(&self, start: usize, key: &K) -> Option<usize> {
        assert_eq!(start, utils::hash_index(key, self.hasher, N));
        IterCollisions::new(key, self.entries, self.hasher).last()
    }

    pub fn remove(self) -> V {
        self.remove_entry().1
    }

    pub fn remove_entry(self) -> (K, V) {
        debug_assert!(*self.len > 0);
        *self.len -= 1;

        if let Some(collision) = self.find_with_hash(self.index, self.key()) {
            self.entries.swap(collision, self.index);

            self.entries[collision].take().unwrap()
        } else {
            self.entries
                .get_mut(self.index)
                //
                .unwrap()
                .take()
                .unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use core::hash::{BuildHasherDefault, Hasher};

    use super::*;
    use pretty_assertions::assert_eq;

    // This hasher will always cause a collision
    #[derive(Default)]
    pub struct CollisionHasher;

    impl Hasher for CollisionHasher {
        fn write(&mut self, _: &[u8]) {}

        fn finish(&self) -> u64 {
            0
        }
    }

    #[test]
    fn test_occupied() {
        let mut entries = [
            Some((0, "a")),
            Some((1, "b")),
            Some((2, "c")),
            Some((3, "d")),
            None,
            None,
        ];

        let build_hasher = BuildHasherDefault::<CollisionHasher>::default();
        let mut len = 4;
        let mut occupied = OccupiedEntry::new(&mut entries, 0, &build_hasher, &mut len);

        assert_eq!(occupied.key(), &0);
        assert_eq!(occupied.get(), &"a");
        assert_eq!(occupied.get_mut(), &mut "a");
    }
}
