mod iter_circular;
mod iter_entries;

pub(crate) use iter_circular::*;
pub(crate) use iter_entries::*;

use core::array;
use core::borrow::Borrow;
use core::hash::{BuildHasher, Hash, Hasher};
use core::mem::MaybeUninit;
use core::ops::Try;

#[must_use]
pub(crate) fn make_hash<K, Q, B>(hash_builder: &B, value: &Q) -> u64
where
    K: Borrow<Q>,
    Q: Hash + ?Sized,
    B: BuildHasher,
{
    let mut hasher = hash_builder.build_hasher();
    value.hash(&mut hasher);
    hasher.finish()
}

#[must_use]
pub(crate) fn adjust_hash<const N: usize>(hash: u64) -> usize {
    (hash % (N as u64)) as usize
}

pub(crate) fn key_hasher<K, V, B>(build_hasher: &B) -> impl FnMut(&(K, V)) -> u64 + '_
where
    B: BuildHasher,
    K: Hash,
{
    move |(k, _)| make_hash::<K, K, B>(build_hasher, k)
}

/// Asserts that `x` is always `true`.
///
/// # Safety
///
/// `x` must be `true`
pub(crate) unsafe fn invariant(x: bool) {
    debug_assert!(x, "invariant does not hold");
    if !x {
        ::core::hint::unreachable_unchecked()
    }
}

/// Converts an `Option<T>` into `T`.
///
/// # Safety
///
/// the `option` must be `Some(T)`
#[must_use]
pub(crate) unsafe fn unwrap_unchecked<T>(option: Option<T>) -> T {
    debug_assert!(option.is_some());

    if let Some(value) = option {
        value
    } else {
        ::core::hint::unreachable_unchecked()
    }
}

pub trait ArrayExt<T, const N: usize> {
    fn enumerate(self) -> [(usize, T); N];

    fn try_map<F, U, R, X>(self, f: F) -> R
    where
        X: Try<Output = U>,
        R: Try<Output = [U; N], Residual = X::Residual>,
        F: FnMut(T) -> X;
}

impl<T, const N: usize> ArrayExt<T, N> for [T; N] {
    fn enumerate(self) -> [(usize, T); N] {
        let mut index = 0;
        self.map(|value| {
            index += 1;
            (index - 1, value)
        })
    }

    fn try_map<F, U, R, X>(self, mut f: F) -> R
    where
        X: Try<Output = U>,
        R: Try<Output = [U; N], Residual = X::Residual>,
        F: FnMut(T) -> X,
    {
        let mut array: [MaybeUninit<U>; N] = MaybeUninit::uninit_array();
        let mut iterator = array::IntoIter::new(self);

        for item in array.iter_mut() {
            // NOTE: it is guranteed that this will not panic
            let next = iterator.next().unwrap();
            *item = MaybeUninit::new(f(next)?);
        }

        // SAFETY: because of the previous loops all values are guranteed to be
        //         initialized
        let result: [U; N] = unsafe { MaybeUninit::array_assume_init(array) };

        R::from_output(result)
    }
}
