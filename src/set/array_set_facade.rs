use core::borrow::Borrow;
use core::hash::{BuildHasher, Hash};

use crate::iter::MapLeftIter;
use crate::raw::{ArrayTable, RawTable, RawTableIter};
use crate::set::{Set, SetIter};
use crate::{ArrayMapFacade, CapacityError, DefaultHashBuilder};

pub type ArraySet<T, const N: usize, B = DefaultHashBuilder> =
    ArraySetFacade<T, ArrayTable<(T, ()), N>, B>;

#[derive(Copy, Clone)]
pub struct ArraySetFacade<T, R: RawTable<(T, ())>, B = DefaultHashBuilder> {
    map: ArrayMapFacade<T, (), R, B>,
}

pub type IntoIter<T, R> = MapLeftIter<(T, ()), <R as IntoIterator>::IntoIter>;

impl<T, R: RawTable<(T, ())> + Default> ArraySetFacade<T, R, DefaultHashBuilder> {
    /// Creates an empty `ArraySetFacade` with the [`DefaultHashBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::set::{ArraySet, DefaultHashBuilder, Set};
    ///
    /// let set: ArraySet<usize, 31, DefaultHashBuilder> = ArraySet::new();
    /// assert_eq!(set.is_empty(), true);
    /// assert_eq!(set.capacity(), 31);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T, R: RawTable<(T, ())>, B: BuildHasher> ArraySetFacade<T, R, B> {
    /// Returns the number of elements the set can hold in total.
    /// The returned value will be equal to the const generic `N`.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::set::{ArraySet, DefaultHashBuilder};
    ///
    /// let set: ArraySet<usize, 31, DefaultHashBuilder> = ArraySet::new();
    /// assert_eq!(set.capacity(), 31);
    /// ```
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.map.capacity()
    }
}

impl<T: Hash + Eq, R: RawTable<(T, ())>, B: BuildHasher> Set<T> for ArraySetFacade<T, R, B> {
    type Error = CapacityError;

    fn len(&self) -> usize {
        self.map.len()
    }

    fn get<Q: ?Sized>(&self, value: &Q) -> Option<&T>
    where
        T: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.map.get_key_value(value).map(|(k, _)| k)
    }

    fn try_replace(&mut self, value: T) -> Result<Option<T>, Self::Error> {
        let old = self.map.remove_entry(&value);

        self.map.try_insert(value, ())?;

        Ok(old.map(|(k, _)| k))
    }

    fn take<Q: ?Sized>(&mut self, value: &Q) -> Option<T>
    where
        T: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.map.remove_entry(&value).map(|(k, _)| k)
    }
}

impl<T, R: RawTable<(T, ())> + Default, B: BuildHasher + Default> Default
    for ArraySetFacade<T, R, B>
{
    fn default() -> Self {
        Self {
            map: ArrayMapFacade::default(),
        }
    }
}

impl<T, R, B> SetIter<T> for ArraySetFacade<T, R, B>
where
    T: Hash + Eq,
    R: RawTableIter<(T, ())>,
    B: BuildHasher,
{
    type Iter<'a>
    where
        T: 'a,
    = crate::iter::Keys<'a, T, (), R>;

    fn iter(&self) -> Self::Iter<'_> {
        self.map.keys()
    }
}

impl<T, R, B> IntoIterator for ArraySetFacade<T, R, B>
where
    T: Hash + Eq,
    R: RawTable<(T, ())>,
    B: BuildHasher,
{
    type IntoIter = IntoIter<T, R>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        MapLeftIter::new(self.map.into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ext::IteratorExt;
    use crate::set::SetIter;
    use pretty_assertions::assert_eq;

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum Key {
        A,
        B,
        C,
        D,
    }

    impl Key {
        pub fn variants() -> impl IntoIterator<Item = Self> {
            [Self::A, Self::B, Self::C, Self::D]
        }
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_is_disjoint_hashset() {
        use std::collections::HashSet;

        let mut set = ArraySet::<u32, 31>::new();
        let mut hash_set = HashSet::<u32>::new();

        for i in 0..10 {
            set.try_insert(i).unwrap();
            hash_set.insert(i);
        }

        assert_eq!(set.is_disjoint(&hash_set), false);

        let mut disjoint = HashSet::<u32>::new();
        for i in 10..30 {
            disjoint.insert(i);
        }

        assert_eq!(set.is_disjoint(&disjoint), true);
    }

    #[test]
    fn test_is_disjoint() {
        // invariant: a.is_disjoint(&b) == b.is_disjoint(&a)
        let mut set = ArraySet::<u32, 31>::new();
        let mut set2 = ArraySet::<u32, 31>::new();

        for i in 0..10 {
            set.try_insert(i).unwrap();
            set2.try_insert(i).unwrap();
        }

        assert_eq!(set.is_disjoint(&set2), false);

        let mut disjoint = ArraySet::<u32, 25>::new();
        for i in 10..30 {
            disjoint.try_insert(i).unwrap();
        }

        assert_eq!(set.is_disjoint(&disjoint), true);
        assert_eq!(set.is_disjoint(&ArraySet::<_, 102>::new()), true);
        assert_eq!(ArraySet::<_, 102>::new().is_disjoint(&set), true);
    }

    #[test]
    fn test_into_iter() {
        let mut set = ArraySet::<Key, 4>::new();

        for variant in Key::variants() {
            set.try_insert(variant).unwrap();
        }

        let mut array: [Option<Key>; 5] = set.into_iter().try_collect().unwrap();
        array.sort();
        assert_eq!(
            array,
            [None, Some(Key::A), Some(Key::B), Some(Key::C), Some(Key::D)]
        );
    }
}
