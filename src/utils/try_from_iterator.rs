use core::iter::IntoIterator;

use crate::utils::ArrayExt;

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
        let mut iterator = iter.into_iter();
        let mut missing = N;

        if let Some(result) = [(); N].try_map(|_| {
            iterator.next().map(|v| {
                missing -= 1;
                v
            })
        }) {
            Ok(result)
        } else {
            Err(CollectArrayError::NotEnoughItems { missing })
        }
    }
}

impl<A, const N: usize> TryFromIterator<A> for [Option<A>; N] {
    type Error = !;

    fn try_from_iter<T: IntoIterator<Item = A>>(iter: T) -> Result<Self, Self::Error> {
        let mut iterator = iter.into_iter();
        Ok([(); N].map(|_| iterator.next()))
    }
}
