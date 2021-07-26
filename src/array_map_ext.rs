use core::hash::{BuildHasher, Hash};

use crate::array_map::ArrayMap;
use crate::errors::CapacityError;
use crate::ext::TryExtend;

pub trait ArrayMapExt<K, V, B: BuildHasher, const N: usize>: Sized
where
    K: Eq + Hash,
{
    fn try_flat_map<X, Y, U, F, const M: usize>(
        self,
        f: F,
    ) -> Result<ArrayMap<X, Y, M, B>, CapacityError>
    where
        F: FnMut(K, V) -> U,
        X: Eq + Hash,
        U: IntoIterator<Item = (X, Y)>;

    #[must_use]
    fn map<X, Y, F>(self, mut f: F) -> ArrayMap<X, Y, N, B>
    where
        F: FnMut(K, V) -> (X, Y),
        X: Eq + Hash,
    {
        self.filter_map(|k, v| Some(f(k, v)))
    }

    #[must_use]
    fn filter_map<X, Y, F>(self, mut f: F) -> ArrayMap<X, Y, N, B>
    where
        F: FnMut(K, V) -> Option<(X, Y)>,
        X: Eq + Hash,
    {
        self.try_flat_map(|k, v| f(k, v)).unwrap()
    }

    #[must_use]
    fn filter<P>(self, mut predicate: P) -> ArrayMap<K, V, N, B>
    where
        P: FnMut(&K, &V) -> bool,
    {
        self.filter_map(|k, v| predicate(&k, &v).then(|| (k, v)))
    }
}

impl<K, V, B: BuildHasher, const N: usize> ArrayMapExt<K, V, B, N> for ArrayMap<K, V, N, B>
where
    K: Eq + Hash,
{
    fn try_flat_map<X, Y, U, F, const M: usize>(
        self,
        mut f: F,
    ) -> Result<ArrayMap<X, Y, M, B>, CapacityError>
    where
        F: FnMut(K, V) -> U,
        X: Eq + Hash,
        U: IntoIterator<Item = (X, Y)>,
    {
        let (build_hasher, entries) = self.into_parts();
        let mut result = ArrayMap::with_build_hasher(build_hasher);

        result.try_extend(entries.flat_map(|(k, v)| f(k, v)))?;

        Ok(result)
    }
}
