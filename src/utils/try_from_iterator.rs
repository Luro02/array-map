use core::iter::IntoIterator;
use core::mem::MaybeUninit;

pub trait TryFromIterator<A>: Sized {
    type Error;

    fn try_from_iter<T: IntoIterator<Item = A>>(iter: T) -> Result<Self, Self::Error>;
}

pub trait IteratorExt: Iterator {
    fn try_collect<B: TryFromIterator<Self::Item>>(self) -> Result<B, B::Error>
    where
        Self: Sized,
    {
        TryFromIterator::try_from_iter(self)
    }
}

impl<I: Iterator> IteratorExt for I {}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CollectArrayError {
    NotEnoughItems { missing: usize },
}

impl<A, const N: usize> TryFromIterator<A> for [A; N] {
    type Error = CollectArrayError;

    fn try_from_iter<T: IntoIterator<Item = A>>(iter: T) -> Result<Self, Self::Error> {
        let mut array: [MaybeUninit<A>; N] = MaybeUninit::uninit_array();
        let mut iterator = iter.into_iter();

        for (i, item) in array.iter_mut().enumerate() {
            if let Some(value) = iterator.next() {
                *item = MaybeUninit::new(value);
            } else {
                return Err(CollectArrayError::NotEnoughItems { missing: N - i });
            }
        }

        let result: [A; N] = unsafe { MaybeUninit::array_assume_init(array) };

        Ok(result)
    }
}

impl<A, const N: usize> TryFromIterator<A> for [Option<A>; N] {
    type Error = !;

    fn try_from_iter<T: IntoIterator<Item = A>>(iter: T) -> Result<Self, Self::Error> {
        let mut iterator = iter.into_iter();
        Ok([(); N].map(|_| iterator.next()))
    }
}
