use core::borrow::Borrow;
use core::hash::{BuildHasher, Hash};

use crate::utils;

pub struct RawEntryBuilder<'a, K, V, B: BuildHasher, const N: usize> {
    entries: &'a [Option<(K, V)>; N],
    build_hasher: &'a B,
}

impl<'a, K, V, B, const N: usize> RawEntryBuilder<'a, K, V, B, N>
where
    B: BuildHasher,
    K: Hash + Eq, // TODO: not required in hashbrown implementation?
{
    #[must_use]
    pub(crate) fn new(entries: &'a [Option<(K, V)>; N], build_hasher: &'a B) -> Self {
        Self {
            entries,
            build_hasher,
        }
    }

    pub fn from_key<Q: ?Sized>(self, qkey: &Q) -> Option<(&'a K, &'a V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let hash = utils::make_hash::<K, Q, B>(self.build_hasher, qkey);

        self.from_key_hashed_nocheck(hash, qkey)
    }

    pub fn from_key_hashed_nocheck<Q: ?Sized>(self, hash: u64, qkey: &Q) -> Option<(&'a K, &'a V)>
    where
        K: Borrow<Q>,
        Q: Eq,
    {
        self.from_hash(hash, |key| key.borrow() == qkey)
    }

    pub fn from_hash<F>(self, hash: u64, mut is_match: F) -> Option<(&'a K, &'a V)>
    where
        F: FnMut(&K) -> bool,
    {
        unimplemented!()
    }
}
