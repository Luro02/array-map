mod iter_circular;

pub(crate) use iter_circular::*;

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

pub(crate) fn key_hasher<K, V, B>(build_hasher: &B) -> impl Fn(&(K, V)) -> u64 + '_
where
    B: BuildHasher,
    K: Hash,
{
    move |(k, _)| make_hash::<K, K, B>(build_hasher, k)
}

#[doc(hidden)]
#[macro_export]
#[cfg(debug_assertions)]
macro_rules! unreachable_unchecked {
    () => {
        ::core::unreachable!()
    };
    ($msg:expr $(,)?) => { ::core::unreachable!($msg) };
    ($fmt:expr, $($arg:tt)*) => { ::core::unreachable!($fmt, $($arg)*) };
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! unreachable_unchecked {
    () => {
        #[allow(unused_unsafe)]
        unsafe {
            ::core::hint::unreachable_unchecked()
        }
    };
    ($msg:expr $(,)?) => {
        #[allow(unused_unsafe)]
        unsafe {
            ::core::hint::unreachable_unchecked()
        }
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[allow(unused_unsafe)]
        unsafe {
            ::core::hint::unreachable_unchecked()
        }
    };
}

/// Asserts that `x` is always `true`.
///
/// # Safety
///
/// `x` must be `true`
#[doc(hidden)]
#[macro_export]
macro_rules! invariant {
    ($x:expr) => {{
        if !($x) {
            $crate::unreachable_unchecked!();
        }
    }};
}

#[cfg(not(feature = "nightly"))]
#[cold]
#[inline]
fn cold() {}

#[inline]
#[must_use]
#[cfg(feature = "nightly")]
pub(crate) fn likely(b: bool) -> bool {
    ::core::intrinsics::likely(b)
}

#[inline]
#[must_use]
#[cfg(feature = "nightly")]
pub(crate) fn unlikely(b: bool) -> bool {
    ::core::intrinsics::unlikely(b)
}

#[inline]
#[must_use]
#[cfg(not(feature = "nightly"))]
pub(crate) fn likely(b: bool) -> bool {
    if !b {
        cold()
    }
    b
}

#[inline]
#[must_use]
#[cfg(not(feature = "nightly"))]
pub(crate) fn unlikely(b: bool) -> bool {
    if b {
        cold()
    }
    b
}

/// Converts an `Option<T>` into `T`.
///
/// # Safety
///
/// the `option` must be `Some(T)`
#[must_use]
pub(crate) unsafe fn unwrap_unchecked<T>(option: Option<T>) -> T {
    if let Some(value) = option {
        value
    } else {
        unreachable_unchecked!()
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
        let mut iterator = core::array::IntoIter::new(self);

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

#[must_use]
pub(crate) fn none_array<T, const N: usize>() -> [Option<T>; N] {
    [(); N].map(|_| None)
}
