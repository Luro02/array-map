use core::borrow::Borrow;
use core::hash::{BuildHasher, Hash};

use crate::utils;
use crate::utils::IterCircular;
use crate::DefaultHashBuilder;

pub struct IterEntries<'a, K: 'a, V: 'a, const N: usize, H: BuildHasher = DefaultHashBuilder> {
    iter: IterCircular<'a, Option<(K, V)>>,
    hasher: &'a H,
    start: usize,
}

impl<'a, K, V, const N: usize, H: BuildHasher> IterEntries<'a, K, V, N, H> {
    pub fn new<Q: ?Sized>(key: &Q, hasher: &'a H, entries: &'a [Option<(K, V)>; N]) -> Self
    where
        Q: Hash + Eq,
        K: Borrow<Q>,
    {
        let start = utils::hash_index(key, hasher, N);

        Self {
            iter: IterCircular::new(start, entries),
            hasher,
            start,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Slot<'a, K: 'a> {
    /// the entry has the same hash
    Collision { index: usize, key: &'a K },
    /// the entry does not have the same hash
    Occupied { index: usize, key: &'a K },
    /// the entry at the index is empty (vacant)
    Vacant { index: usize },
}

impl<'a, K: Hash, V, H: BuildHasher, const N: usize> Iterator for IterEntries<'a, K, V, N, H> {
    type Item = Slot<'a, K>;

    fn next(&mut self) -> Option<Self::Item> {
        let (index, entry) = self.iter.next()?;

        if let Some((key, _)) = entry {
            // check if the current entry is a collision:
            if utils::hash_index(key, self.hasher, N) == self.start {
                Some(Slot::Collision { index, key })
            } else {
                Some(Slot::Occupied { index, key })
            }
        } else {
            Some(Slot::Vacant { index })
        }
    }
}

impl<'a, K, V, B, const N: usize> DoubleEndedIterator for IterEntries<'a, K, V, N, B>
where
    K: Hash,
    B: BuildHasher,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let (index, entry) = self.iter.next_back()?;

        if let Some((key, _)) = entry {
            // check if the current entry is a collision:
            if utils::hash_index(key, self.hasher, N) == self.start {
                Some(Slot::Collision { index, key })
            } else {
                Some(Slot::Occupied { index, key })
            }
        } else {
            Some(Slot::Vacant { index })
        }
    }
}
