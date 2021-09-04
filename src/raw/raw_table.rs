use core::mem;

use crate::ext::ToIter;
use crate::UnavailableMutError;

pub trait RawTable<T>: IntoIterator<Item = T> {
    /// A type that uniquely identifes an occupied entry in the table.
    ///
    /// This ident must be valid until a value has been removed from the table.
    type Ident: Clone;
    type InsertError;
    type RawIter: Iterator<Item = Self::Ident>;
    type DrainIter: Iterator<Item = T>;

    /// Searches for an entry in the table with the given hash. If there are
    /// multiple entries with the same hash, the eq method is used to
    /// determine which one to return.
    #[must_use]
    fn find(&self, hash: u64, eq: impl FnMut(&T) -> bool) -> Option<Self::Ident>;

    /// Returns a shared reference to the entry with the provided identifier.
    ///
    /// # Safety
    ///
    /// After removing any value from the table, it is no longer guranteed that
    /// existing identifier point to occupied entries.
    #[must_use]
    unsafe fn get_unchecked(&self, ident: Self::Ident) -> &T;

    /// Returns a mutable reference to the entry with the provided identifier.
    ///
    /// # Safety
    ///
    /// After removing any value from the table, it is no longer guranteed that
    /// existing identifier point to occupied entries.
    #[must_use]
    unsafe fn get_unchecked_mut(&mut self, ident: Self::Ident) -> &mut T;

    /// Removes an entry from the table without invalidating existing
    /// identifiers.
    ///
    /// # Safety
    ///
    /// This may cause entries to become unreachable through `find`.
    unsafe fn erase(&mut self, ident: Self::Ident) -> T;

    #[must_use]
    fn drain(&mut self) -> Self::DrainIter;

    /// Inserts the value in the table and returns the identifier which
    /// represents the value.
    ///
    /// # Note
    ///
    /// This method does not check if the value already exists in the table,
    /// which could cause it to be overwritten or having the same value multiple
    /// times in the table.
    ///
    /// # Errors
    ///
    /// If there is not enough space in the table (`self.capacity() ==
    /// self.len()`)
    fn try_insert(
        &mut self,
        hash: u64,
        value: T,
        hasher: impl Fn(&T) -> u64,
    ) -> Result<Self::Ident, Self::InsertError>;

    #[must_use]
    fn iter(&self) -> Self::RawIter;

    /// Removes the entry associated with the ident from the table and returns
    /// it's value.
    ///
    /// # Safety
    ///
    /// One must ensure that the `ident` is still valid.
    unsafe fn remove(&mut self, ident: Self::Ident, hasher: impl Fn(&T) -> u64) -> T;

    /// Returns the number of entries that can be inserted in the table.
    ///
    /// # Note
    ///
    /// Depending on the table it may error or resize if needed.
    #[must_use]
    fn capacity(&self) -> usize;

    #[must_use]
    fn get_each_mut<const M: usize>(
        &mut self,
        hashes: [u64; M],
        eq: impl FnMut(usize, &T) -> bool,
    ) -> [Result<&mut T, UnavailableMutError>; M];

    /// Clears the table, which removes all entries.
    fn clear(&mut self) {
        mem::drop(self.drain());
    }

    /// Returns the number of entries that are currently in the table.
    #[must_use]
    fn len(&self) -> usize {
        self.iter().count()
    }

    /// Returns `true` if the table is empty, `false` if it is not empty.
    #[must_use]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[must_use]
    fn get(&self, hash: u64, eq: impl FnMut(&T) -> bool) -> Option<&T> {
        if let Some(ident) = self.find(hash, eq) {
            // SAFETY: remove has not been called after the ident has been created
            Some(unsafe { self.get_unchecked(ident) })
        } else {
            None
        }
    }

    #[must_use]
    fn get_mut(&mut self, hash: u64, eq: impl FnMut(&T) -> bool) -> Option<&mut T> {
        if let Some(ident) = self.find(hash, eq) {
            // SAFETY: remove has not been called after the ident has been created
            Some(unsafe { self.get_unchecked_mut(ident) })
        } else {
            None
        }
    }

    /// This method is used to rediscover lost entries (can be caused by
    /// `erase`) in the table.
    fn rehash(&mut self, hasher: impl Fn(&T) -> u64) {
        for entry in self.drain() {
            mem::drop(self.try_insert(hasher(&entry), entry, |value| hasher(value)));
        }
    }
}

pub trait RawTableIter<T>: RawTable<T> {
    type IterMut<'a>: ToIter<T> + Iterator<Item = &'a mut T>
    where
        T: 'a;
    type Iter<'a>: Iterator<Item = &'a T>
    where
        T: 'a;

    /// Returns a mutable iterator over the table.
    fn iter_mut(&mut self) -> Self::IterMut<'_>;

    /// Returns an immutable iterator over the table.
    fn iter(&self) -> Self::Iter<'_>;
}

/// A trait implemented by [`RawTable`]s with a fixed capacity.
pub trait FixedSizeTable<T, const N: usize>: RawTable<T> {}
