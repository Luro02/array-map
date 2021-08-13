use core::borrow::Borrow;
use core::hash::{BuildHasher, Hash};
use core::marker::PhantomData;
use core::ops::Index;
use core::{fmt, mem};

use crate::entry::Entry;
use crate::errors::{CapacityError, RescaleError, UnavailableMutError};
use crate::ext::{TryExtend, TryFromIterator};
use crate::iter::{Drain, DrainFilter, Iter, IterMut, Keys, Values, ValuesMut};
use crate::occupied::OccupiedEntry;
use crate::raw::{ArrayTable, RawEntryBuilder, RawTable, RawTableIter};
use crate::utils;
use crate::vacant::VacantEntry;

/// Default hasher for [`ArrayMapFacade`].
#[cfg(feature = "ahash")]
pub type DefaultHashBuilder = core::hash::BuildHasherDefault<ahash::AHasher>;
/// Dummy default hasher
#[cfg(not(feature = "ahash"))]
pub enum DefaultHashBuilder {}

pub type ArrayMap<K, V, const N: usize, B = DefaultHashBuilder> =
    ArrayMapFacade<K, V, ArrayTable<(K, V), N>, B>;

#[derive(Copy, Clone)]
pub struct ArrayMapFacade<K, V, R: RawTable<(K, V)>, B = DefaultHashBuilder> {
    table: R,
    build_hasher: B,
    _p: PhantomData<(K, V)>,
}

#[cfg(feature = "ahash")]
impl<K, V, R: RawTable<(K, V)> + Default> ArrayMapFacade<K, V, R, DefaultHashBuilder> {
    /// Creates an empty [`ArrayMapFacade`] with the [`DefaultHashBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::{ArrayMap, DefaultHashBuilder};
    ///
    /// let mut map: ArrayMap<usize, &str, 31, DefaultHashBuilder> = ArrayMap::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::with_hasher(DefaultHashBuilder::default())
    }
}

impl<K, V, R: RawTable<(K, V)> + Default, B: BuildHasher> ArrayMapFacade<K, V, R, B> {
    /// Creates an empty [`ArrayMapFacade`] with the provided [`BuildHasher`].
    ///
    /// # Note
    ///
    /// This is function is identical to [`ArrayMap::with_build_hasher`], the
    /// only difference is the name of the function. This function exists
    /// for API compatibility with the standard librarys `HashMap`.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::{ArrayMap, DefaultHashBuilder};
    ///
    /// let build_hasher = DefaultHashBuilder::default();
    /// let mut map: ArrayMap<usize, &str, 31, DefaultHashBuilder> =
    ///     ArrayMap::with_hasher(build_hasher);
    /// ```
    #[must_use]
    #[doc(alias("with_build_hasher"))]
    pub fn with_hasher(build_hasher: B) -> Self {
        Self::with_build_hasher(build_hasher)
    }

    /// Creates an empty [`ArrayMapFacade`] with the provided [`BuildHasher`].
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::{ArrayMap, DefaultHashBuilder};
    ///
    /// let build_hasher = DefaultHashBuilder::default();
    /// let mut map: ArrayMap<usize, &str, 31, DefaultHashBuilder> =
    ///     ArrayMap::with_build_hasher(build_hasher);
    /// ```
    #[must_use]
    #[doc(alias("with_hasher"))]
    pub fn with_build_hasher(build_hasher: B) -> Self {
        Self {
            table: R::default(),
            build_hasher,
            _p: PhantomData,
        }
    }
}

impl<K, V, R: RawTable<(K, V)>, B: BuildHasher> ArrayMapFacade<K, V, R, B> {
    /// Returns the number of elements the map can hold in total.
    ///
    /// The returned value, will be equal to the const generic `N`.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// let map: ArrayMap<i32, i32, 2> = ArrayMap::new();
    /// assert_eq!(map.capacity(), 2);
    ///
    /// let map: ArrayMap<&str, usize, 1234> = ArrayMap::new();
    /// assert_eq!(map.capacity(), 1234);
    /// ```
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.table.capacity()
    }

    /// Returns the number of elements in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// let mut map: ArrayMap<u16, &str, 3> = ArrayMap::new();
    ///
    /// assert_eq!(map.len(), 0);
    /// map.insert(1, "a");
    /// assert_eq!(map.len(), 1);
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Returns `true` if the map contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// let mut map: ArrayMap<_, _, 3> = ArrayMap::new();
    ///
    /// assert_eq!(map.is_empty(), true);
    ///
    /// map.insert(1, "a")?;
    /// assert_eq!(map.len(), 1);
    /// assert_eq!(map.is_empty(), false);
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a reference to the map's [`BuildHasher`].
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::{ArrayMap, DefaultHashBuilder};
    ///
    /// let hasher = DefaultHashBuilder::default();
    /// let map: ArrayMap<i32, i32, 12> = ArrayMap::with_hasher(hasher);
    /// let hasher: &DefaultHashBuilder = map.build_hasher();
    /// ```
    #[must_use]
    #[doc(alias("hasher"))]
    pub fn build_hasher(&self) -> &B {
        &self.build_hasher
    }
}

impl<K, V, R, B> ArrayMapFacade<K, V, R, B>
where
    K: Eq + Hash,
    R: RawTable<(K, V)>,
    B: BuildHasher,
{
    /// Gets the given key's corresponding entry in the map for in-place
    /// manipulation.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// // there are exactly 26 letters in the english alphabet
    /// let mut letters: ArrayMap<char, u32, 26> = ArrayMap::new();
    ///
    /// // c = character
    /// for c in "a short treatise on fungi".chars() {
    ///     let counter = letters.entry(c)?.or_insert(0);
    ///     *counter += 1;
    /// }
    ///
    /// assert_eq!(letters[&'s'], 2);
    /// assert_eq!(letters[&'t'], 3);
    /// assert_eq!(letters[&'u'], 1);
    /// assert_eq!(letters.get(&'y'), None);
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    pub fn entry(&mut self, key: K) -> Result<Entry<'_, K, V, R, B>, CapacityError> {
        let hash = utils::make_hash::<K, K, B>(&self.build_hasher, &key);

        if let Some(ident) = self.table.find(hash, |(k, _)| k.eq(&key)) {
            unsafe {
                Ok(Entry::Occupied(OccupiedEntry::new(
                    &mut self.table,
                    ident,
                    &self.build_hasher,
                )))
            }
        } else {
            if self.table.len() == self.table.capacity() {
                Err(CapacityError)
            } else {
                unsafe {
                    Ok(Entry::Vacant(VacantEntry::new(
                        &mut self.table,
                        key,
                        &self.build_hasher,
                    )))
                }
            }
        }
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, [`None`] is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old
    /// value is returned. The key is not updated, though; this matters for
    /// types that can be `==` without being identical. See the [module-level
    /// documentation] for more.
    ///
    /// [`None`]: https://doc.rust-lang.org/std/option/enum.Option.html#variant.None
    /// [module-level documentation]: index.html#insert-and-complex-keys
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// #[derive(Debug, Clone, PartialEq)]
    /// enum Status {
    ///     Occupied,
    ///     Empty,
    /// }
    ///
    /// let mut castles: ArrayMap<&str, Status, 59> = ArrayMap::new();
    ///
    /// // as you can see the status of the "Shimada Castle" is currently unknown:
    /// assert_eq!(castles.get("Shimada Castle"), None);
    ///
    /// let castle_entry = castles.entry("Shimada Castle")?;
    /// assert_eq!(castle_entry.insert(Status::Occupied), None);
    /// assert_eq!(castles["Shimada Castle"], Status::Occupied);
    ///
    /// // you can also overwrite existing castles:
    /// castles.insert("Anvil Castle", Status::Occupied);
    ///
    /// let castle_entry = castles.entry("Anvil Castle")?;
    /// assert_eq!(castle_entry.insert(Status::Empty), Some(Status::Occupied));
    /// assert_eq!(castles["Anvil Castle"], Status::Empty);
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>, CapacityError> {
        Ok(self.entry(key)?.insert(value))
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// let mut map: ArrayMap<i32, &str, 13> = ArrayMap::new();
    /// map.insert(1, "a")?;
    ///
    /// assert_eq!(map.get(&1), Some(&"a"));
    /// assert_eq!(map.get(&2), None);
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    #[must_use]
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.get_key_value(key).map(|(_, v)| v)
    }

    /// Returns a mutable reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// let mut map: ArrayMap<i32, &str, 13> = ArrayMap::new();
    ///
    /// map.insert(1, "a")?;
    ///
    /// if let Some(x) = map.get_mut(&1) {
    ///     *x = "b";
    /// }
    /// assert_eq!(map[&1], "b");
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    #[must_use]
    pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.get_key_value_mut(key).map(|(_, v)| v)
    }

    /// Clears the map, removing all key-value pairs.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// let mut map: ArrayMap<u32, &str, 3> = ArrayMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.is_empty(), false);
    ///
    /// map.clear();
    /// assert_eq!(map.is_empty(), true);
    /// ```
    pub fn clear(&mut self) {
        self.table.clear();
    }

    /// Returns `true` if the map contains a value for the specified key.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// let mut map: ArrayMap<i32, &str, 3> = ArrayMap::new();
    ///
    /// map.insert(1, "a")?;
    /// assert_eq!(map.contains_key(&1), true);
    /// assert_eq!(map.contains_key(&2), false);
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    pub fn contains_key<Q: ?Sized>(&self, qkey: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.get(qkey).is_some()
    }

    /// Returns the key-value pair corresponding to the supplied key.
    ///
    /// The supplied key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// let mut map: ArrayMap<i32, &str, 13> = ArrayMap::new();
    /// map.insert(1, "a")?;
    ///
    /// assert_eq!(map.get_key_value(&1), Some((&1, &"a")));
    /// assert_eq!(map.get_key_value(&2), None);
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    pub fn get_key_value<Q: ?Sized>(&self, key: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let hash = utils::make_hash::<K, Q, B>(&self.build_hasher, key);
        self.table
            .get(hash, |(k, _)| key.eq(k.borrow()))
            .map(|(k, v)| (k, v))
    }

    /// Returns the key-value pair corresponding to the supplied key.
    ///
    /// The supplied key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// let mut map: ArrayMap<i32, &str, 13> = ArrayMap::new();
    /// map.insert(1, "a")?;
    ///
    /// let (k, v) = map.get_key_value_mut(&1).unwrap();
    /// assert_eq!(k, &1);
    /// assert_eq!(v, &mut "a");
    /// *v = "b";
    /// assert_eq!(map.get_key_value_mut(&1), Some((&1, &mut "b")));
    /// assert_eq!(map.get_key_value_mut(&2), None);
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    pub fn get_key_value_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<(&K, &mut V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let hash = utils::make_hash::<K, Q, B>(&self.build_hasher, key);

        match self.table.get_mut(hash, move |(k, _)| key.eq(k.borrow())) {
            Some((k, v)) => Some((k, v)),
            None => None,
        }
    }

    /// Removes a key from the map, returning the stored key and value if the
    /// key was previously in the map.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// let mut map: ArrayMap<i32, &str, 3> = ArrayMap::new();
    ///
    /// map.insert(1, "a")?;
    /// assert_eq!(map.remove_entry(&1), Some((1, "a")));
    /// assert_eq!(map.remove_entry(&1), None);
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    pub fn remove_entry<Q: ?Sized>(&mut self, qkey: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let hash = utils::make_hash::<K, Q, B>(&self.build_hasher, &qkey);
        unsafe {
            if let Some(ident) = self.table.find(hash, |(k, _)| qkey.eq(k.borrow())) {
                let entry = self
                    .table
                    .remove(ident, utils::key_hasher(&self.build_hasher));
                Some(entry)
            } else {
                None
            }
        }
    }

    /// Removes a key from the map, returning the value at the key if the key
    /// was previously in the map.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// let mut map: ArrayMap<i32, &str, 3> = ArrayMap::new();
    ///
    /// map.insert(1, "a")?;
    /// assert_eq!(map.remove(&1), Some("a"));
    /// assert_eq!(map.remove(&1), None);
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    pub fn remove<Q: ?Sized>(&mut self, qkey: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.remove_entry(qkey).map(|(_, v)| v)
    }

    /// Attempts to get mutable references to `M` values in the map at once,
    /// with immutable references to the corresponding keys.
    ///
    /// Returns an array of length `M` with the results of each query. For
    /// soundness, at most one mutable reference will be returned to any
    /// value. An `Err(UnavailableMutError::Duplicate(i))` in the returned
    /// array indicates that a suitable key-value pair exists, but a mutable
    /// reference to the value already occurs at index `i` in the returned
    /// array.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::{ArrayMap, UnavailableMutError};
    ///
    /// let mut libraries: ArrayMap<&str, usize, 19> = ArrayMap::new();
    ///
    /// libraries.insert("Bodleian Library", 1602)?;
    /// libraries.insert("Athenæum", 1807)?;
    /// libraries.insert("Herzogin-Anna-Amalia-Bibliothek", 1691)?;
    /// libraries.insert("Library of Congress", 1800)?;
    ///
    /// let got = libraries.get_each_key_value_mut([
    ///     "Bodleian Library",
    ///     "Herzogin-Anna-Amalia-Bibliothek",
    ///     "Herzogin-Anna-Amalia-Bibliothek",
    ///     "Gewandhaus",
    /// ]);
    ///
    /// assert_eq!(
    ///     got,
    ///     [
    ///         Ok((&"Bodleian Library", &mut 1602)),
    ///         Ok((&"Herzogin-Anna-Amalia-Bibliothek", &mut 1691)),
    ///         Err(UnavailableMutError::Duplicate(1)),
    ///         Err(UnavailableMutError::Absent),
    ///     ]
    /// );
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    pub fn get_each_key_value_mut<Q: ?Sized, const M: usize>(
        &mut self,
        qkeys: [&Q; M],
    ) -> [Result<(&K, &mut V), UnavailableMutError>; M]
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.table
            .get_each_mut(
                qkeys.map(|key| utils::make_hash::<K, Q, B>(&self.build_hasher, key)),
                |index, (key, _)| qkeys[index].eq(key.borrow()),
            )
            .map(|entry| {
                let (key, value) = entry?;
                Ok((&*key, value))
            })
    }

    /// Attempts to get mutable references to `N` values in the map at once.
    ///
    /// Returns an array of length `N` with the results of each query. For
    /// soundness, at most one mutable reference will be returned to any
    /// value. An `Err(UnavailableMutError::Duplicate(i))` in the returned
    /// array indicates that a suitable key-value pair exists, but a mutable
    /// reference to the value already occurs at index `i` in the returned
    /// array.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::{ArrayMap, UnavailableMutError};
    ///
    /// let mut libraries: ArrayMap<&str, usize, 19> = ArrayMap::new();
    /// libraries.insert("Bodleian Library", 1602)?;
    /// libraries.insert("Athenæum", 1807)?;
    /// libraries.insert("Herzogin-Anna-Amalia-Bibliothek", 1691)?;
    /// libraries.insert("Library of Congress", 1800)?;
    ///
    /// let got = libraries.get_each_value_mut([
    ///     "Athenæum",
    ///     "New York Public Library",
    ///     "Athenæum",
    ///     "Library of Congress",
    /// ]);
    /// assert_eq!(
    ///     got,
    ///     [
    ///         Ok(&mut 1807),
    ///         Err(UnavailableMutError::Absent),
    ///         Err(UnavailableMutError::Duplicate(0)),
    ///         Ok(&mut 1800),
    ///     ]
    /// );
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    #[doc(alias("get_each_mut"))]
    pub fn get_each_value_mut<Q: ?Sized, const M: usize>(
        &mut self,
        qkeys: [&Q; M],
    ) -> [Result<&mut V, UnavailableMutError>; M]
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.get_each_key_value_mut(qkeys).map(|entry| Ok(entry?.1))
    }

    /// Creates an iterator which uses a closure to determine if an element
    /// should be removed.
    ///
    /// If the closure returns `true`, the element is removed from the map and
    /// yielded. If the closure returns `false`, or panics, the element
    /// remains in the map and will not be yielded.
    ///
    /// Note that `drain_filter` lets you mutate every value in the filter
    /// closure, regardless of whether you choose to keep or remove it.
    ///
    /// If the iterator is only partially consumed or not consumed at all, each
    /// of the remaining elements will still be subjected to the closure and
    /// removed and dropped if it returns true.
    ///
    /// It is unspecified how many more elements will be subjected to the
    /// closure if a panic occurs in the closure, or a panic occurs while
    /// dropping an element, or if the `DrainFilter` value is leaked.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ext::IteratorExt;
    /// use array_map::{array_map, ArrayMap};
    ///
    /// let mut map = array_map! {
    ///     "hello" => "hallo",
    ///     "world" => "welt",
    ///     "apple" => "apfel",
    ///     "rust" => "rost",
    /// };
    ///
    /// let mut drained: [Option<(_, _)>; 4] =
    ///     map.drain_filter(|k, v| k.len() < 5).try_collect().unwrap();
    /// drained.sort_unstable();
    ///
    /// assert_eq!(drained, [None, None, None, Some(("rust", "rost")),]);
    /// ```
    pub fn drain_filter<F>(&mut self, f: F) -> DrainFilter<'_, K, V, F, R, B>
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        DrainFilter::new(f, &mut self.table, &self.build_hasher)
    }

    /// Clears the map, returning all key-value pairs as an iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ext::IteratorExt;
    /// use array_map::{array_map, ArrayMap};
    ///
    /// let mut map = array_map! {
    ///     "hello" => "hallo",
    ///     "world" => "welt",
    ///     "apple" => "apfel",
    /// };
    ///
    /// let mut drained: [Option<(_, _)>; 3] = map.drain().try_collect().unwrap();
    /// drained.sort_unstable();
    ///
    /// assert_eq!(
    ///     drained,
    ///     [
    ///         Some(("apple", "apfel")),
    ///         Some(("hello", "hallo")),
    ///         Some(("world", "welt")),
    ///     ]
    /// );
    /// ```
    pub fn drain(&mut self) -> Drain<'_, K, V, R, B> {
        Drain::new(&mut self.table, &self.build_hasher)
    }

    /// Tries to convert the map with capacity `N` into a map with capacity `M`.
    ///
    /// # Errors
    ///
    /// An error will be returned, if the length of the map ([`ArrayMap::len`])
    /// is larger than `M`.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// let mut map: ArrayMap<char, u32, 1> = ArrayMap::new();
    ///
    /// map.insert('a', 'a' as u32)?;
    ///
    /// // no more values can be inserted
    ///
    /// let rescaled: ArrayMap<_, _, 3> = map.try_rescale().expect("failed to rescale");
    ///
    /// assert_eq!(rescaled.len(), 1);
    /// assert_eq!(rescaled.capacity(), 3);
    /// assert_eq!(rescaled.get(&'a'), Some(&('a' as u32)));
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    pub fn try_rescale<const M: usize>(mut self) -> Result<ArrayMap<K, V, M, B>, RescaleError<M>> {
        if self.len() >= M {
            return Err(RescaleError::new(self.len(), self.capacity()));
        }

        let mut result = ArrayMap::with_build_hasher(self.build_hasher);

        for (key, value) in self.table.drain() {
            // explicitly ignore the result, because it can not fail (has been checked
            // before the loop)
            mem::drop(result.insert(key, value));
        }

        Ok(result)
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// Calls the provided function on each entry, removing all entries,
    /// where the function returns `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// // maps the chars 'a' to 'z' and '0' to '9' to their numerical value
    /// let mut map: ArrayMap<char, u32, { 26 + 10 }> = ArrayMap::new();
    ///
    /// for c in 'a'..='z' {
    ///     map.insert(c, c as u32)?;
    /// }
    ///
    /// for c in '0'..='9' {
    ///     map.insert(c, c as u32)?;
    /// }
    ///
    /// // only keep chars that are not digits:
    /// map.retain(|c, _| !c.is_ascii_digit());
    ///
    /// for c in 'a'..='z' {
    ///     assert_eq!(map.get(&c), Some(&(c as u32)));
    /// }
    ///
    /// for c in '0'..='9' {
    ///     assert_eq!(map.get(&c), None);
    /// }
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        self.drain_filter(|key, value| !(f(key, value)));
    }

    /// Creates a raw immutable entry builder for the `ArrayMap`.
    ///
    /// Raw entries provide the lowest level of control for searching and
    /// manipulating a map.
    ///
    /// They must be manually initialized with a hash and then manually
    /// searched.
    ///
    /// This is useful for
    /// - Hash memoization
    /// - Using a search key that does not work with the [`Borrow`] trait
    /// - Using custom comparison logic with newtype wrappers
    ///
    /// Unless you are in such a situation, higher-level and more foolproof APIs
    /// like [`ArrayMap::get`] should be preferred.
    ///
    /// Immutable raw entries have a very limited use; you might instead want to
    /// use `ArrayMap::raw_entry_mut`.
    #[must_use]
    pub fn raw_entry(&self) -> RawEntryBuilder<'_, K, V, R, B> {
        RawEntryBuilder::new(&self.table, &self.build_hasher)
    }

    pub(crate) fn into_parts(self) -> (B, <R as IntoIterator>::IntoIter) {
        (self.build_hasher, self.table.into_iter())
    }
}

impl<K, V, R, B> ArrayMapFacade<K, V, R, B>
where
    R: RawTableIter<(K, V)>,
    B: BuildHasher,
{
    /// Returns an iterator iterating over the immutable entries of the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ext::IteratorExt;
    /// use array_map::{array_map, ArrayMap};
    /// # use array_map::ext::CollectArrayError;
    ///
    /// let mut map: ArrayMap<&str, &str, 3> = array_map! {
    ///     "hello" => "你好",
    ///     "good night" => "晚安",
    ///     "good bye" => "再見",
    /// };
    ///
    /// let mut iterated: [(&&str, &&str); 3] = map.iter().try_collect()?;
    /// iterated.sort_unstable();
    ///
    /// assert_eq!(
    ///     iterated,
    ///     [
    ///         (&"good bye", &"再見"),
    ///         (&"good night", &"晚安"),
    ///         (&"hello", &"你好"),
    ///     ]
    /// );
    /// # Ok::<_, CollectArrayError>(())
    /// ```
    pub fn iter(&self) -> Iter<'_, K, V, R> {
        Iter::new(&self.table)
    }

    /// Returns an iterator iterating over the mutable entries of the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::{array_map, ArrayMap};
    ///
    /// let mut map = array_map! {
    ///    0 => 1,
    ///    1 => 2,
    ///    2 => 3,
    /// };
    ///
    /// for (key, value) in map.iter_mut() {
    ///     if key % 2 == 0 {
    ///         *value *= 2;
    ///     } else {
    ///         *value += 5;
    ///     }
    /// }
    ///
    /// assert_eq!(
    ///     map,
    ///     array_map! {
    ///        0 => 2,
    ///        1 => 7,
    ///        2 => 6,
    ///     }
    /// );
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V, R> {
        IterMut::new(&mut self.table)
    }

    /// An iterator visiting all keys in arbitrary order.
    /// The iterator element type is `&'a K`.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ext::IteratorExt;
    /// use array_map::{array_map, ArrayMap};
    /// # use array_map::ext::CollectArrayError;
    ///
    /// let mut map: ArrayMap<&str, &str, 3> = array_map! {
    ///     "hello" => "salut",
    ///     "good night" => "bonne nuit",
    ///     "good bye" => "au revoir",
    /// };
    ///
    /// let mut keys: [&&str; 3] = map.keys().try_collect()?;
    /// keys.sort_unstable();
    ///
    /// assert_eq!(keys, [&"good bye", &"good night", &"hello",]);
    /// # Ok::<_, CollectArrayError>(())
    /// ```
    pub fn keys(&self) -> Keys<'_, K, V, R> {
        Keys::new(self.iter())
    }

    /// An iterator visiting all values in arbitrary order.
    /// The iterator element type is `&'a V`.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ext::IteratorExt;
    /// use array_map::{array_map, ArrayMap};
    /// # use array_map::ext::CollectArrayError;
    ///
    /// let mut map: ArrayMap<&str, &str, 3> = array_map! {
    ///     "hello" => "salut",
    ///     "good night" => "bonne nuit",
    ///     "good bye" => "au revoir",
    /// };
    ///
    /// let mut values: [&&str; 3] = map.values().try_collect()?;
    /// values.sort_unstable();
    ///
    /// assert_eq!(values, [&"au revoir", &"bonne nuit", &"salut",]);
    /// # Ok::<_, CollectArrayError>(())
    /// ```
    pub fn values(&self) -> Values<'_, K, V, R> {
        Values::new(self.iter())
    }

    /// An iterator visiting all values in arbitrary order.
    /// The iterator element type is `&mut V`.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ext::IteratorExt;
    /// use array_map::{array_map, ArrayMap};
    /// # use array_map::ext::CollectArrayError;
    ///
    /// let mut map = array_map! {
    ///    0 => 1,
    ///    1 => 2,
    ///    2 => 3,
    /// };
    ///
    /// for value in map.values_mut() {
    ///     if *value % 2 == 0 {
    ///         *value += 1;
    ///     }
    /// }
    ///
    /// assert_eq!(
    ///     map,
    ///     array_map! {
    ///        0 => 1,
    ///        1 => 3,
    ///        2 => 3,
    ///     }
    /// );
    /// ```
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V, R> {
        ValuesMut::new(self.iter_mut())
    }
}

impl<K, Q: ?Sized, V, R, B> Index<&Q> for ArrayMapFacade<K, V, R, B>
where
    K: Eq + Hash + Borrow<Q>,
    Q: Eq + Hash,
    R: RawTable<(K, V)>,
    B: BuildHasher,
{
    type Output = V;

    /// Returns a reference to the value corresponding to the supplied key.
    ///
    /// # Panics
    ///
    /// Panics if the key is not present in the `ArrayMap`.
    fn index(&self, key: &Q) -> &V {
        self.get(key).expect("no entry found for key")
    }
}

impl<'a, K, V, R, B> IntoIterator for &'a ArrayMapFacade<K, V, R, B>
where
    R: RawTableIter<(K, V)>,
    B: BuildHasher,
{
    type IntoIter = Iter<'a, K, V, R>;
    type Item = (&'a K, &'a V);

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V, R, B> IntoIterator for &'a mut ArrayMapFacade<K, V, R, B>
where
    R: RawTableIter<(K, V)>,
    B: BuildHasher,
{
    type IntoIter = IterMut<'a, K, V, R>;
    type Item = (&'a K, &'a mut V);

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<K, V, R, B> IntoIterator for ArrayMapFacade<K, V, R, B>
where
    R: RawTable<(K, V)>,
    B: BuildHasher,
{
    type IntoIter = <R as IntoIterator>::IntoIter;
    type Item = (K, V);

    fn into_iter(self) -> Self::IntoIter {
        self.table.into_iter()
    }
}

impl<K, V, R, B> Default for ArrayMapFacade<K, V, R, B>
where
    R: RawTable<(K, V)> + Default,
    B: BuildHasher + Default,
{
    fn default() -> Self {
        Self::with_hasher(B::default())
    }
}

impl<K, V, R, B> fmt::Debug for ArrayMapFacade<K, V, R, B>
where
    K: fmt::Debug,
    V: fmt::Debug,
    R: RawTableIter<(K, V)>,
    B: BuildHasher,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<K, V, R, B> PartialEq<Self> for ArrayMapFacade<K, V, R, B>
where
    K: Eq + Hash,
    V: PartialEq,
    R: RawTableIter<(K, V)>,
    B: BuildHasher,
{
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter()
            .all(|(key, value)| other.get(key).map_or(false, |v| *value == *v))
    }
}

impl<K, V, R, B> Eq for ArrayMapFacade<K, V, R, B>
where
    K: Eq + Hash,
    V: PartialEq,
    R: RawTableIter<(K, V)>,
    B: BuildHasher,
{
}

impl<K, V, R, B> TryFromIterator<(K, V)> for ArrayMapFacade<K, V, R, B>
where
    K: Eq + Hash,
    R: RawTable<(K, V)> + Default,
    B: BuildHasher + Default,
{
    type Error = CapacityError;

    fn try_from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Result<Self, Self::Error> {
        let mut result = Self::with_build_hasher(B::default());

        for (key, value) in iter {
            result.insert(key, value)?;
        }

        Ok(result)
    }
}

impl<K, V, R, B> TryExtend<(K, V)> for ArrayMapFacade<K, V, R, B>
where
    K: Eq + Hash,
    R: RawTable<(K, V)>,
    B: BuildHasher,
{
    type Error = CapacityError;

    fn try_extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) -> Result<(), Self::Error> {
        for (key, value) in iter {
            self.insert(key, value)?;
        }

        Ok(())
    }
}

impl<'a, K, V, R, B> TryExtend<(&'a K, &'a V)> for ArrayMapFacade<K, V, R, B>
where
    K: Eq + Hash + Copy,
    V: Copy,
    R: RawTable<(K, V)>,
    B: BuildHasher,
{
    type Error = CapacityError;

    fn try_extend<T: IntoIterator<Item = (&'a K, &'a V)>>(
        &mut self,
        iter: T,
    ) -> Result<(), Self::Error> {
        self.try_extend(iter.into_iter().map(|(k, v)| (k, v)))
    }
}

#[cfg(all(test, feature = "ahash"))]
mod tests {
    use crate::array_map;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_insert() {
        let mut map = ArrayMap::<_, _, 2>::new();

        assert_eq!(map.insert("hello", "world"), Ok(None));
        assert_eq!(map.insert("other", "world2"), Ok(None));
        assert_eq!(map.get("hello"), Some(&"world"));
        assert_eq!(map.get("other"), Some(&"world2"));
    }

    // TODO: better test!
    #[test]
    fn test_get_each_mut() {
        let mut map: ArrayMap<_, _, 19> = ArrayMap::new();
        map.insert("foo", 0).unwrap();
        map.insert("bar", 10).unwrap();
        map.insert("baz", 20).unwrap();
        map.insert("qux", 30).unwrap();

        let ys = map.get_each_key_value_mut(["bar", "baz", "baz", "dip"]);
        assert_eq!(
            ys,
            [
                Ok((&"bar", &mut 10)),
                Ok((&"baz", &mut 20)),
                Err(UnavailableMutError::Duplicate(1)),
                Err(UnavailableMutError::Absent),
            ]
        );
    }

    #[derive(PartialEq, Eq)]
    struct HasHash(u64, u64);

    impl core::hash::Hash for HasHash {
        fn hash<H>(&self, h: &mut H)
        where
            H: core::hash::Hasher,
        {
            h.write_u64(self.0);
        }
    }
    impl fmt::Debug for HasHash {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "HasHash({}, {})", self.0, self.1)
        }
    }

    #[derive(Default)]
    struct Hasher(u64);

    impl core::hash::Hasher for Hasher {
        fn finish(&self) -> u64 {
            self.0
        }

        fn write(&mut self, _: &[u8]) {
            unimplemented!()
        }

        fn write_u64(&mut self, value: u64) {
            self.0 = value;
        }
    }

    #[test]
    fn test_remove_shift_with_smaller_hash() {
        let mut map: ArrayMap<HasHash, usize, 5, _> = array_map! {
            @infer,
            @build_hasher => ::core::hash::BuildHasherDefault::<Hasher>::default(),
            HasHash(0, 0) => 0,
            HasHash(1, 0) => 1,
            HasHash(2, 0) => 2,
            HasHash(0, 1) => 3,
        }
        .unwrap();

        assert_eq!(
            map.table,
            ArrayTable::from_array([
                Some((HasHash(0, 0), 0)),
                Some((HasHash(1, 0), 1)),
                Some((HasHash(2, 0), 2)),
                Some((HasHash(0, 1), 3)),
                None,
            ])
        );

        assert!(map.contains_key(&HasHash(0, 1)));
        assert_eq!(map.remove(&HasHash(1, 0)), Some(1));
        assert_eq!(
            map.table,
            ArrayTable::from_array([
                Some((HasHash(0, 0), 0)),
                Some((HasHash(0, 1), 3)),
                Some((HasHash(2, 0), 2)),
                None,
                None,
            ])
        );
        assert!(map.contains_key(&HasHash(0, 1)));
    }

    #[test]
    fn test_linear_probing_move_after_remove() {
        let mut map: ArrayMap<HasHash, usize, 2, _> = array_map! {
            @infer,
            @build_hasher => ::core::hash::BuildHasherDefault::<Hasher>::default(),
            HasHash(1, 1) => 1,
            HasHash(0, 0) => 0,
        }
        .unwrap();

        assert_eq!(
            map.table,
            ArrayTable::from_array([
                //
                Some((HasHash(0, 0), 0)),
                Some((HasHash(1, 1), 1)),
            ])
        );

        assert_eq!(map.remove(&HasHash(1, 1)), Some(1));
        assert_eq!(
            map.table,
            ArrayTable::from_array([
                //
                Some((HasHash(0, 0), 0)),
                None,
            ])
        );

        assert!(map.contains_key(&HasHash(0, 0)));
    }

    #[test]
    fn test_linear_probing_swap_after_remove() {
        let mut map: ArrayMap<HasHash, usize, 4, _> = array_map! {
            @infer,
            @build_hasher => ::core::hash::BuildHasherDefault::<Hasher>::default(),
            HasHash(1, 1) => 1,
            HasHash(0, 2) => 0,
            HasHash(0, 0) => 2,
        }
        .unwrap();

        assert_eq!(
            map.table,
            ArrayTable::from_array([
                Some((HasHash(0, 2), 0)), // 0
                Some((HasHash(1, 1), 1)), // 1
                Some((HasHash(0, 0), 2)), // 2
                None,
            ])
        );

        assert_eq!(map.remove(&HasHash(0, 2)), Some(0));
        assert_eq!(
            map.table,
            ArrayTable::from_array([
                //
                Some((HasHash(0, 0), 2)),
                Some((HasHash(1, 1), 1)),
                None,
                None,
            ])
        );
    }

    #[test]
    fn test_linear_probing_remove_no_move() {
        let mut map: ArrayMap<HasHash, usize, 4, _> = array_map! {
            @infer,
            @build_hasher => ::core::hash::BuildHasherDefault::<Hasher>::default(),
            HasHash(1, 1) => 1,
            HasHash(0, 2) => 0,
            HasHash(0, 0) => 2,
        }
        .unwrap();

        assert_eq!(
            map.table,
            ArrayTable::from_array([
                Some((HasHash(0, 2), 0)), // 0
                Some((HasHash(1, 1), 1)), // 1
                Some((HasHash(0, 0), 2)), // 2
                None,
            ])
        );

        assert_eq!(map.remove(&HasHash(0, 0)), Some(2));
        assert_eq!(
            map.table,
            ArrayTable::from_array([
                //
                Some((HasHash(0, 2), 0)),
                Some((HasHash(1, 1), 1)),
                None,
                None,
            ])
        );
    }

    #[test]
    fn test_linear_probing_stop_index() {
        let mut map: ArrayMap<HasHash, usize, 5, _> = array_map! {
            @infer,
            @build_hasher => ::core::hash::BuildHasherDefault::<Hasher>::default(),
            HasHash(2, 0) => 0,
            HasHash(4, 1) => 1,
            HasHash(4, 2) => 2,
            HasHash(2, 3) => 4,
            HasHash(0, 0) => 5,
        }
        .unwrap();

        assert_eq!(
            map.table,
            ArrayTable::from_array([
                Some((HasHash(4, 2), 2)), // 0
                Some((HasHash(0, 0), 5)), // 1
                Some((HasHash(2, 0), 0)), // 2
                Some((HasHash(2, 3), 4)), // 3
                Some((HasHash(4, 1), 1)), // 4
            ])
        );

        assert_eq!(map.remove(&HasHash(2, 0)), Some(0));
        assert_eq!(
            map.table,
            ArrayTable::from_array([
                //
                Some((HasHash(4, 2), 2)), // 0
                Some((HasHash(0, 0), 5)), // 1
                Some((HasHash(2, 3), 4)), // 2
                None,                     // 3
                Some((HasHash(4, 1), 1)), // 4
            ])
        );

        assert_eq!(map.remove(&HasHash(4, 1)), Some(1));
        assert_eq!(
            map.table,
            ArrayTable::from_array([
                //
                Some((HasHash(0, 0), 5)), // 0
                None,                     // 1
                Some((HasHash(2, 3), 4)), // 2
                None,                     // 3
                Some((HasHash(4, 2), 2)), // 4
            ])
        );
    }

    #[test]
    fn test_fuzzer_failure_00() {
        let mut map: ArrayMap<HasHash, usize, 4, _> = array_map! {
            @infer,
            @build_hasher => ::core::hash::BuildHasherDefault::<Hasher>::default(),
            HasHash(1, 1) => 0,
            HasHash(0, 1) => 1,
            HasHash(0, 0) => 2,
        }
        .unwrap();

        assert_eq!(
            map.table,
            ArrayTable::from_array([
                Some((HasHash(0, 1), 1)), // 0
                Some((HasHash(1, 1), 0)), // 1
                Some((HasHash(0, 0), 2)), // 2
                None,
            ])
        );

        assert_eq!(map.remove(&HasHash(1, 1)), Some(0));

        assert_eq!(
            map.table,
            ArrayTable::from_array([
                Some((HasHash(0, 1), 1)), // 0
                Some((HasHash(0, 0), 2)), // 2
                None,
                None,
            ])
        );
    }

    #[test]
    fn test_fuzzer_failure_01() {
        let mut map: ArrayMap<HasHash, usize, 4, _> = array_map! {
            @infer,
            @build_hasher => ::core::hash::BuildHasherDefault::<Hasher>::default(),
            HasHash(3, 1) => 0,
            HasHash(2, 1) => 1,
            HasHash(2, 0) => 2,
        }
        .unwrap();

        assert_eq!(
            map.table,
            ArrayTable::from_array([
                Some((HasHash(2, 0), 2)), // 0
                None,                     // 1
                Some((HasHash(2, 1), 1)), // 2
                Some((HasHash(3, 1), 0)), // 3
            ])
        );

        assert_eq!(map.remove(&HasHash(3, 1)), Some(0));

        assert_eq!(
            map.table,
            ArrayTable::from_array([
                None,                     // 0
                None,                     // 1
                Some((HasHash(2, 1), 1)), // 2
                Some((HasHash(2, 0), 2)), // 3
            ])
        );
    }

    #[test]
    fn test_fuzzer_failure_02() {
        let mut map: ArrayMap<HasHash, usize, 4, _> = array_map! {
            @infer,
            @build_hasher => ::core::hash::BuildHasherDefault::<Hasher>::default(),
            HasHash(2, 0) => 0,
            HasHash(2, 1) => 1,
            HasHash(3, 2) => 2,
        }
        .unwrap();

        assert_eq!(
            map.table,
            ArrayTable::from_array([
                Some((HasHash(3, 2), 2)), // 0
                None,                     // 1
                Some((HasHash(2, 0), 0)), // 2
                Some((HasHash(2, 1), 1)), // 3
            ])
        );

        assert_eq!(map.remove(&HasHash(2, 0)), Some(0));

        assert_eq!(
            map.table,
            ArrayTable::from_array([
                None,                     // 0
                None,                     // 1
                Some((HasHash(2, 1), 1)), // 2
                Some((HasHash(3, 2), 2)), // 3
            ])
        );
    }

    #[test]
    fn test_fuzzer_failure_03() {
        let mut map: ArrayMap<HasHash, usize, 4, _> = array_map! {
            @infer,
            @build_hasher => ::core::hash::BuildHasherDefault::<Hasher>::default(),
            HasHash(3, 1) => 0,
            HasHash(3, 3) => 1,
            HasHash(0, 0) => 2,
        }
        .unwrap();

        assert_eq!(
            map.table,
            ArrayTable::from_array([
                Some((HasHash(3, 3), 1)), // 0
                Some((HasHash(0, 0), 2)), // 1
                None,                     // 2
                Some((HasHash(3, 1), 0)), // 3
            ])
        );

        assert_eq!(map.remove(&HasHash(3, 1)), Some(0));

        assert_eq!(
            map.table,
            ArrayTable::from_array([
                Some((HasHash(0, 0), 2)), // 0
                None,                     // 1
                None,                     // 2
                Some((HasHash(3, 3), 1)), // 3
            ])
        );
    }

    #[test]
    fn test_fuzzer_failure_04() {
        let mut map: ArrayMap<HasHash, usize, 4, _> = array_map! {
            @infer,
            @build_hasher => ::core::hash::BuildHasherDefault::<Hasher>::default(),
            HasHash(0, 0) => 0,
            HasHash(3, 1) => 1,
            HasHash(3, 2) => 2,
        }
        .unwrap();

        assert_eq!(
            map.table,
            ArrayTable::from_array([
                Some((HasHash(0, 0), 0)), // 0
                Some((HasHash(3, 2), 2)), // 1
                None,                     // 2
                Some((HasHash(3, 1), 1)), // 3
            ])
        );

        assert_eq!(map.remove(&HasHash(0, 0)), Some(0));

        assert_eq!(
            map.table,
            ArrayTable::from_array([
                Some((HasHash(3, 2), 2)), // 0
                None,                     // 1
                None,                     // 2
                Some((HasHash(3, 1), 1)), // 3
            ])
        );
    }
}
