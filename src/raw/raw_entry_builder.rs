use core::borrow::Borrow;
use core::fmt;
use core::hash::{BuildHasher, Hash};
use core::marker::PhantomData;

use crate::raw::RawTable;
use crate::utils;

pub struct RawEntryBuilder<'a, K, V, R: RawTable<(K, V)>, B: BuildHasher> {
    table: &'a R,
    build_hasher: &'a B,
    _p: PhantomData<&'a (K, V)>,
}

impl<'a, K, V, R, B> RawEntryBuilder<'a, K, V, R, B>
where
    R: RawTable<(K, V)>,
    B: BuildHasher,
{
    #[must_use]
    pub(crate) fn new(table: &'a R, build_hasher: &'a B) -> Self {
        Self {
            table,
            build_hasher,
            _p: PhantomData,
        }
    }
}

impl<'a, K, V, R, B> RawEntryBuilder<'a, K, V, R, B>
where
    B: BuildHasher,
    R: RawTable<(K, V)>,
{
    /// Access an entry by key.
    pub fn from_key<Q: ?Sized>(self, qkey: &Q) -> Option<(&'a K, &'a V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let hash = utils::make_hash::<K, Q, B>(self.build_hasher, qkey);

        self.from_key_hashed_nocheck(hash, qkey)
    }

    /// Access an entry by a key and its hash.
    pub fn from_key_hashed_nocheck<Q: ?Sized>(self, hash: u64, qkey: &Q) -> Option<(&'a K, &'a V)>
    where
        K: Borrow<Q>,
        Q: Eq,
    {
        self.from_hash(hash, |key| key.borrow() == qkey)
    }

    /// Access an entry by hash.
    pub fn from_hash<F>(self, hash: u64, is_match: F) -> Option<(&'a K, &'a V)>
    where
        F: FnMut(&K) -> bool,
    {
        self.search(hash, is_match)
    }

    fn search<F>(self, hash: u64, mut is_match: F) -> Option<(&'a K, &'a V)>
    where
        F: FnMut(&K) -> bool,
    {
        match self.table.get(hash, |(key, _)| is_match(key)) {
            Some((key, value)) => Some((key, value)),
            None => None,
        }
    }
}

impl<'a, K, V, R, B> fmt::Debug for RawEntryBuilder<'a, K, V, R, B>
where
    R: RawTable<(K, V)>,
    B: BuildHasher,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!(RawEntryBuilder)).finish()
    }
}
