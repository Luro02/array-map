use core::borrow::Borrow;
use core::hash::Hash;

use crate::set::iter::{Difference, Intersection, SymmetricDifference, Union};

/// A trait that is implemented by structs that are sets.
///
/// So one can for example check if a `HashSet` and an `ArraySet` are disjoint.
pub trait Set<T> {
    type Error;

    /// Returns the number of elements in the set.
    #[must_use]
    fn len(&self) -> usize;

    /// Returns a reference to the value in the set, if any, that is equal to
    /// the given value.
    ///
    /// The value may be any borrowed form of the set's value type, but [`Hash`]
    /// and [`Eq`] on the borrowed form **must** match those for the value
    /// type.
    #[must_use]
    fn get<Q: ?Sized>(&self, value: &Q) -> Option<&T>
    where
        T: Borrow<Q>,
        Q: Hash + Eq;

    /// Adds a value to the set, replacing the existing value, if any, that is
    /// equal to the given one. Returns the replaced value.
    ///
    /// # Errors
    ///
    /// If the set is full or it failed to insert the value.
    fn try_replace(&mut self, value: T) -> Result<Option<T>, Self::Error>;

    /// Removes and returns the value in the set, if any, that is equal to the
    /// given one.
    ///
    /// The value may be any borrowed form of the set's value type, but [`Hash`]
    /// and [`Eq`] on the borrowed form **must** match those for the value
    /// type.
    fn take<Q: ?Sized>(&mut self, value: &Q) -> Option<T>
    where
        T: Borrow<Q>,
        Q: Hash + Eq;

    /// Adds the value to the set.
    ///
    /// If the set did not have this value present, `true` is returned.
    ///
    /// If the set did have this value present, `false` is returned.
    ///
    /// # Errors
    ///
    /// If the set is full or it failed to insert the value.
    fn try_insert(&mut self, value: T) -> Result<bool, Self::Error>
    where
        T: Hash + Eq,
    {
        if self.contains(&value) {
            return Ok(false);
        }

        self.try_replace(value).map(|_| true)
    }

    /// Removes a value from the set.
    ///
    /// Returns `true` if the value was present in the set and `false` if it was
    /// not in the set.
    fn remove<Q: ?Sized>(&mut self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.take(value).is_some()
    }

    /// Returns true if the value is in the set, false if it is not.
    #[must_use]
    fn contains<Q: ?Sized>(&self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.get(value).is_some()
    }

    /// Returns `true` if the set is empty, `false` if it is not empty.
    #[must_use]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait SetIter<T>: Set<T> + IntoIterator<Item = T> {
    /// An immutable iterator over the elements of the set.
    type Iter<'a>: Iterator<Item = &'a T>
    where
        T: 'a;

    /// An iterator visiting all elements in arbitrary order. The iterator
    /// element type is `&'a T`.
    fn iter(&self) -> Self::Iter<'_>;

    /// Visits the values representing the difference, i.e., the values that are
    /// in `self` but not in `other`.
    ///
    /// In mathematical notation this is `self \ other` or `{ x | x ∈ self ∧ x ∉
    /// other }`

    fn difference<'a, S: Set<T>>(&'a self, other: &'a S) -> Difference<'a, T, Self, S>
    where
        Self: Sized,
    {
        Difference::new(self, other)
    }

    /// Visits the values representing the symmetric difference, i.e., the
    /// values that are in self or in other but not in both.
    ///
    /// In mathematical notation this is `self ∪ other \ self ∩ other`
    fn symmetric_difference<'a, S: SetIter<T>>(
        &'a self,
        other: &'a S,
    ) -> SymmetricDifference<'a, T, Self, S>
    where
        Self: Sized,
        T: Hash + Eq,
    {
        SymmetricDifference::new(self, other)
    }

    fn intersection<'a, S: Set<T>>(&'a self, other: &'a S) -> Intersection<'a, T, Self, S>
    where
        Self: Sized,
        T: Hash + Eq,
    {
        Intersection::new(self, other)
    }

    fn union<'a, S: SetIter<T>>(&'a self, other: &'a S) -> Union<'a, T, Self, S>
    where
        Self: Sized,
        T: Hash + Eq,
    {
        Union::new(self, other)
    }

    /// Returns `true` if `self` and `other` have no elements in common.
    ///
    /// This is equivalent to checking for an empty intersection.
    ///
    /// In mathematical notation this is `self ∩ other = ∅` or `(∀x ∈ self: x ∉
    /// other) ∧ (∀x ∈ other: x ∉ self)`
    #[must_use]
    fn is_disjoint<S: SetIter<T>>(&self, other: &S) -> bool
    where
        T: Hash + Eq,
    {
        if self.len() <= other.len() {
            self.iter().all(|v| !other.contains(v))
        } else {
            other.iter().all(|v| !self.contains(v))
        }
    }

    /// Returns `true` if the set is a subset of another, i.e., `other` contains
    /// at least all the values in `self`.
    ///
    /// In mathematical notation this is `self ⊆ other`.
    #[must_use]
    fn is_subset<S: Set<T>>(&self, other: &S) -> bool
    where
        T: Hash + Eq,
    {
        for element in self.iter() {
            if !other.contains(element) {
                return false;
            }
        }

        true
    }

    /// Returns `true` if the set is a superset of another, i.e., `self`
    /// contains at least all the values in `other`.
    ///
    /// In mathematical notation this is `other ⊆ self`.
    #[must_use]
    fn is_superset<S: SetIter<T>>(&self, other: &S) -> bool
    where
        T: Hash + Eq,
        Self: Sized,
    {
        other.is_subset(self)
    }
}

#[cfg(feature = "std")]
impl<T, S> Set<T> for ::std::collections::HashSet<T, S>
where
    T: Hash + Eq,
    S: ::core::hash::BuildHasher,
{
    type Error = !;

    fn len(&self) -> usize {
        Self::len(self)
    }

    fn get<Q: ?Sized>(&self, value: &Q) -> Option<&T>
    where
        T: Borrow<Q>,
        Q: Hash + Eq,
    {
        Self::get(self, value)
    }

    fn try_replace(&mut self, value: T) -> Result<Option<T>, Self::Error> {
        Ok(Self::replace(self, value))
    }

    fn take<Q: ?Sized>(&mut self, value: &Q) -> Option<T>
    where
        T: Borrow<Q>,
        Q: Hash + Eq,
    {
        Self::take(self, value)
    }
}

#[cfg(feature = "std")]
impl<T, S> SetIter<T> for ::std::collections::HashSet<T, S>
where
    T: Hash + Eq,
    S: ::core::hash::BuildHasher,
{
    type Iter<'a>
    where
        T: 'a,
    = ::std::collections::hash_set::Iter<'a, T>;

    fn iter(&self) -> Self::Iter<'_> {
        Self::iter(self)
    }
}
