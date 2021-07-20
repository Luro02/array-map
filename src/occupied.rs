use core::hash::{BuildHasher, Hash};
use core::mem;

use crate::utils::{self, IterEntries, Slot};

#[derive(Debug)]
pub struct OccupiedEntry<'a, K: 'a, V: 'a, B, const N: usize> {
    entries: &'a mut [Option<(K, V)>; N],
    index: usize,
    build_hasher: &'a B,
    len: &'a mut usize,
}

impl<'a, K, V, B: BuildHasher, const N: usize> OccupiedEntry<'a, K, V, B, N> {
    #[must_use]
    pub(crate) fn new(
        entries: &'a mut [Option<(K, V)>; N],
        index: usize,
        build_hasher: &'a B,
        len: &'a mut usize,
    ) -> Self {
        debug_assert_eq!(entries.len(), N);
        debug_assert!(index < N);
        debug_assert!(entries[index].is_some());
        debug_assert!(*len > 0);

        Self {
            entries,
            index,
            build_hasher,
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

trait DoubleEndedIteratorExt: DoubleEndedIterator {
    fn rfind_map<B, F>(&mut self, f: F) -> Option<B>
    where
        F: FnMut(Self::Item) -> Option<B>,
    {
        self.filter_map(f).next_back()
    }
}

impl<D: DoubleEndedIterator> DoubleEndedIteratorExt for D {}

impl<'a, K: Hash + Eq, V, B: BuildHasher, const N: usize> OccupiedEntry<'a, K, V, B, N> {
    fn find_with_hash(&self, key: &K) -> Option<usize> {
        let hash = utils::make_hash::<K, K, B>(&self.build_hasher, key);

        IterEntries::new(hash, self.entries, utils::key_hasher(self.build_hasher)).rfind_map(
            |slot| {
                if let Slot::Collision { index, .. } = slot {
                    Some(index)
                } else {
                    None
                }
            },
        )
    }

    pub fn remove(self) -> V {
        self.remove_entry().1
    }

    pub fn remove_entry(self) -> (K, V) {
        debug_assert!(*self.len > 0);
        *self.len -= 1;

        if let Some(collision) = self.find_with_hash(self.key()) {
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
