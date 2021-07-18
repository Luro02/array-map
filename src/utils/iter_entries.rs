use core::borrow::Borrow;
use core::hash::{BuildHasher, Hash};

use crate::utils;
use crate::utils::IterCircular;
use crate::DefaultHashBuilder;

pub struct IterEntries<'a, K: 'a, V: 'a, const N: usize, H: BuildHasher = DefaultHashBuilder> {
    iter: IterCircular<'a, Option<(K, V)>>,
    hasher: &'a H,
    hash: u64,
}

impl<'a, K, V, const N: usize, H: BuildHasher> IterEntries<'a, K, V, N, H> {
    pub fn new<Q: ?Sized>(key: &Q, hasher: &'a H, entries: &'a [Option<(K, V)>; N]) -> Self
    where
        Q: Hash + Eq,
        K: Borrow<Q>,
    {
        let hash = utils::make_hash::<K, Q, H>(hasher, key);

        Self::new_with_hash(hash, hasher, entries)
    }

    pub(crate) fn new_with_hash(
        hash: u64,
        hasher: &'a H,
        entries: &'a [Option<(K, V)>; N],
    ) -> Self {
        Self {
            iter: IterCircular::new((hash as usize) % N, entries),
            hasher,
            hash,
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
            if utils::make_hash::<K, K, H>(self.hasher, key) == self.hash {
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
            if utils::make_hash::<K, K, B>(self.hasher, key) == self.hash {
                Some(Slot::Collision { index, key })
            } else {
                Some(Slot::Occupied { index, key })
            }
        } else {
            Some(Slot::Vacant { index })
        }
    }
}
