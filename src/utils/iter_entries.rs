use crate::utils;
use crate::utils::IterCircular;

pub(crate) struct IterEntries<'a, T, F: FnMut(&T) -> u64, const N: usize> {
    iter: IterCircular<'a, Option<T>>,
    hasher: F,
    hash: u64,
}

impl<'a, T, F: FnMut(&T) -> u64, const N: usize> IterEntries<'a, T, F, N> {
    pub(crate) fn new(hash: u64, slice: &'a [Option<T>; N], hasher: F) -> Self {
        let iter = IterCircular::new(utils::adjust_hash::<N>(hash), slice);
        Self { iter, hasher, hash }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Slot<'a, T> {
    /// the entry has the same hash
    Collision { index: usize, entry: &'a T },
    /// the entry does not have the same hash
    Occupied { index: usize, entry: &'a T },
    /// the entry at the index is empty (vacant)
    Vacant { index: usize },
}

fn create_slot<T>(
    hash: u64,
    index: usize,
    entry: Option<&T>,
    hasher: impl FnOnce(&T) -> u64,
) -> Slot<'_, T> {
    if let Some(entry) = entry {
        // check if the current entry is a collision:
        if (hasher)(entry) == hash {
            Slot::Collision { index, entry }
        } else {
            Slot::Occupied { index, entry }
        }
    } else {
        Slot::Vacant { index }
    }
}

impl<'a, T, F: FnMut(&T) -> u64, const N: usize> Iterator for IterEntries<'a, T, F, N> {
    type Item = Slot<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let (index, entry) = self.iter.next()?;

        Some(create_slot(
            self.hash,
            index,
            entry.as_ref(),
            &mut self.hasher,
        ))
    }
}

impl<'a, T, F: FnMut(&T) -> u64, const N: usize> DoubleEndedIterator for IterEntries<'a, T, F, N> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let (index, entry) = self.iter.next_back()?;

        Some(create_slot(
            self.hash,
            index,
            entry.as_ref(),
            &mut self.hasher,
        ))
    }
}
