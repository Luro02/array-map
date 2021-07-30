use crate::utils::{self, IterCircular};

pub(crate) struct IterEntries<'a, T, F: FnMut(&T) -> u64, const N: usize> {
    iter: IterCircular<'a, Option<T>>,
    hasher: F,
    hash: u64,
}

impl<'a, T, F: FnMut(&T) -> u64, const N: usize> IterEntries<'a, T, F, N> {
    pub(crate) fn new(hash: u64, slice: &'a [Option<T>; N], hasher: F) -> Self {
        Self::new_with_start(utils::adjust_hash::<N>(hash), hash, slice, hasher)
    }

    pub(crate) fn new_with_start(
        start: usize,
        hash: u64,
        slice: &'a [Option<T>; N],
        hasher: F,
    ) -> Self {
        Self {
            iter: IterCircular::new(start, slice),
            hasher,
            hash,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Slot<'a, T> {
    /// the entry has the same hash
    Collision {
        index: usize,
        entry: &'a T,
        hash: u64,
    },
    /// the entry does not have the same hash
    Occupied {
        index: usize,
        entry: &'a T,
        hash: u64,
    },
    /// the entry at the index is empty (vacant)
    Vacant { index: usize },
}

impl<'a, T> Slot<'a, T> {
    #[must_use]
    pub fn index(&self) -> usize {
        let (Self::Collision { index, .. } | Self::Occupied { index, .. } | Self::Vacant { index }) =
            self;

        *index
    }
}

fn create_slot<T, F, const N: usize>(
    expected_hash: u64,
    index: usize,
    entry: Option<&T>,
    hasher: F,
) -> Slot<'_, T>
where
    F: FnOnce(&T) -> u64,
{
    if let Some(entry) = entry {
        // check if the current entry is a collision:
        let hash = (hasher)(entry);
        let hash_index = super::adjust_hash::<N>(hash);

        if hash_index == super::adjust_hash::<N>(expected_hash) {
            Slot::Collision { index, entry, hash }
        } else {
            Slot::Occupied { index, entry, hash }
        }
    } else {
        Slot::Vacant { index }
    }
}

impl<'a, T, F: FnMut(&T) -> u64, const N: usize> Iterator for IterEntries<'a, T, F, N> {
    type Item = Slot<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let (index, entry) = self.iter.next()?;

        Some(create_slot::<_, _, N>(
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

        Some(create_slot::<_, _, N>(
            self.hash,
            index,
            entry.as_ref(),
            &mut self.hasher,
        ))
    }
}
