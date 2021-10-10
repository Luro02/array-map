use core::iter::FusedIterator;
use core::marker::PhantomData;
use core::{fmt, iter};

use crate::ext::ToIter;

/// This trait is used to generalize over heterogeneous pairs.
pub trait PairLike {
    type Left;
    type Right;

    #[must_use]
    fn left(self) -> Self::Left;

    #[must_use]
    fn right(self) -> Self::Right;

    #[must_use]
    fn borrow(&self) -> (&Self::Left, &Self::Right);
}

impl<K, V> PairLike for (K, V) {
    type Left = K;
    type Right = V;

    #[inline]
    fn left(self) -> Self::Left {
        self.0
    }

    #[inline]
    fn right(self) -> Self::Right {
        self.1
    }

    #[inline]
    fn borrow(&self) -> (&Self::Left, &Self::Right) {
        (&self.0, &self.1)
    }
}

pub struct MapLeftIter<P: PairLike, I> {
    iter: I,
    _p: PhantomData<P>,
}

impl<P: PairLike, I> MapLeftIter<P, I> {
    #[must_use]
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            _p: PhantomData,
        }
    }
}

impl<P, I> Iterator for MapLeftIter<P, I>
where
    I: Iterator<Item = P>,
    P: PairLike,
{
    type Item = P::Left;

    #[allow(clippy::manual_map)]
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(pair) => Some(pair.left()),
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<P, I> DoubleEndedIterator for MapLeftIter<P, I>
where
    I: DoubleEndedIterator<Item = P>,
    P: PairLike,
{
    #[allow(clippy::manual_map)]
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.iter.next_back() {
            Some(pair) => Some(pair.left()),
            None => None,
        }
    }
}

impl<P: PairLike, I: FusedIterator<Item = P>> FusedIterator for MapLeftIter<P, I> {}

impl<P: PairLike, I: ExactSizeIterator<Item = P>> ExactSizeIterator for MapLeftIter<P, I> {}

impl<P, I> Clone for MapLeftIter<P, I>
where
    I: Clone,
    P: PairLike,
{
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            _p: PhantomData,
        }
    }
}

impl<P, I> fmt::Debug for MapLeftIter<P, I>
where
    P: PairLike,
    P::Left: fmt::Debug,
    I: ToIter<Item = P>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

type BorrowPairLikeIter<'a, P, I> =
    iter::Map<I, for<'r> fn(&'r P) -> (&'r <P as PairLike>::Left, &'r <P as PairLike>::Right)>;

fn borrow_pair_like_iter<'a, P: 'a, I>(iter: I) -> BorrowPairLikeIter<'a, P, I>
where
    P: PairLike,
    I: Iterator<Item = &'a P>,
{
    iter.map(PairLike::borrow)
}

impl<P, I> ToIter for MapLeftIter<P, I>
where
    P: PairLike,
    I: ToIter<Item = P>,
{
    type Item = P::Left;
    type Iter<'b>
    where
        Self::Item: 'b,
        Self: 'b,
    = MapLeftIter<(&'b P::Left, &'b P::Right), BorrowPairLikeIter<'b, P, I::Iter<'b>>>;

    fn iter(&self) -> Self::Iter<'_> {
        MapLeftIter::new(borrow_pair_like_iter(self.iter.iter()))
    }
}
