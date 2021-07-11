use core::borrow::Borrow;
use core::fmt;
use core::hash::{BuildHasher, BuildHasherDefault, Hash};
use core::ops::Index;

use ahash::AHasher;

use crate::entry::Entry;
use crate::iter::{Drain, DrainFilter, IntoIter, Iter, IterMut, Keys, Values, ValuesMut};
use crate::occupied::OccupiedEntry;
use crate::utils::Slot;
use crate::utils::{ArrayExt, IterEntries};
use crate::vacant::VacantEntry;

pub type DefaultHashBuilder = BuildHasherDefault<AHasher>;

#[derive(Copy, Clone)]
pub struct ArrayMap<K, V, const N: usize, B: BuildHasher = DefaultHashBuilder> {
    entries: [Option<(K, V)>; N],
    build_hasher: B,
    len: usize,
}

impl<K, V, const N: usize> ArrayMap<K, V, N, DefaultHashBuilder> {
    /// Creates an empty [`ArrayMap`] with the [`DefaultHashBuilder`].
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

impl<K, V, const N: usize, B: BuildHasher> ArrayMap<K, V, N, B> {
    /// Creates an empty [`ArrayMap`] with the provided [`BuildHasher`].
    ///
    /// # Note
    ///
    /// This is function is identical to [`ArrayMap::with_build_hasher`], the only difference
    /// is the name of the function. This function exists for API compatibility with the
    /// standard librarys `HashMap`.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::{ArrayMap, DefaultHashBuilder};
    ///
    /// let build_hasher = DefaultHashBuilder::default();
    /// let mut map: ArrayMap<usize, &str, 31, DefaultHashBuilder> = ArrayMap::with_hasher(build_hasher);
    /// ```
    #[must_use]
    #[doc(alias("with_build_hasher"))]
    pub fn with_hasher(build_hasher: B) -> Self {
        Self::with_build_hasher(build_hasher)
    }

    /// Creates an empty [`ArrayMap`] with the provided [`BuildHasher`].
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::{ArrayMap, DefaultHashBuilder};
    ///
    /// let build_hasher = DefaultHashBuilder::default();
    /// let mut map: ArrayMap<usize, &str, 31, DefaultHashBuilder> = ArrayMap::with_build_hasher(build_hasher);
    /// ```
    #[must_use]
    #[doc(alias("with_hasher"))]
    pub fn with_build_hasher(build_hasher: B) -> Self {
        Self {
            entries: [(); N].map(|_| None),
            build_hasher,
            len: 0,
        }
    }
}

#[must_use]
enum FindResult<T> {
    Vacant(T),
    Occupied(T),
    End,
}

impl<T> FindResult<T> {
    pub fn occupied(self) -> Option<T> {
        match self {
            Self::Occupied(value) => Some(value),
            Self::End | Self::Vacant(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OccupiedError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapacityError;

impl fmt::Display for CapacityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "not enough space")
    }
}

impl<K, V, B, const N: usize> ArrayMap<K, V, N, B>
where
    K: Eq + Hash,
    B: BuildHasher,
{
    /// Gets the given key's corresponding entry in the map for in-place manipulation.
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
    pub fn entry(&mut self, key: K) -> Result<Entry<'_, K, V, N, B>, CapacityError> {
        match self.find(&key) {
            FindResult::Occupied(index) => Ok(Entry::Occupied(OccupiedEntry::new(
                &mut self.entries,
                index,
                &self.build_hasher,
                &mut self.len,
            ))),
            FindResult::Vacant(index) => Ok(Entry::Vacant(VacantEntry::new(
                key,
                &mut self.entries,
                index,
                &self.build_hasher,
                &mut self.len,
            ))),
            FindResult::End => Err(CapacityError), // TODO: when and why does this happen??? (document)
        }
    }

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
        N
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
        self.len
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
    /// [`Eq`]: core::cmp::Eq
    /// [`Hash`]: core::hash::Hash
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
    /// [`Eq`]: core::cmp::Eq
    /// [`Hash`]: core::hash::Hash
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
        self.drain();
    }

    /// Returns `true` if the map contains a value for the specified key.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// [`Eq`]: core::cmp::Eq
    /// [`Hash`]: core::hash::Hash
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
    /// [`Eq`]: core::cmp::Eq
    /// [`Hash`]: core::hash::Hash
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
        let index = self.find(key).occupied()?;

        self.entries[index].as_ref().map(|(k, v)| (k, v))
    }

    /// Returns the key-value pair corresponding to the supplied key.
    ///
    /// The supplied key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// [`Eq`]: core::cmp::Eq
    /// [`Hash`]: core::hash::Hash
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
        let index = self.find(key).occupied()?;

        match self.entries[index].as_mut() {
            Some((k, v)) => Some((k, v)),
            None => None,
        }
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
        Some(self.occupied_entry(qkey)?.remove_entry())
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

    /// Attempts to get mutable references to `M` values in the map at once, with immutable
    /// references to the corresponding keys.
    ///
    /// Returns an array of length `M` with the results of each query. For soundness,
    /// at most one mutable reference will be returned to any value. An
    /// `Err(UnavailableMutError::Duplicate(i))` in the returned array indicates that a suitable
    /// key-value pair exists, but a mutable reference to the value already occurs at index `i` in
    /// the returned array.
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
        // if an entry is already borrowed then an index will be present, which points to the mutable
        // reference in the resulting array
        let mut borrowed: [Option<usize>; N] = [(); N].map(|_| None);
        let qkeys = qkeys.map(|qkey| self.find(qkey).occupied());

        let mut entries = self.entries.each_mut().map(|entry| Some(entry));

        qkeys.enumerate().map(|(idx, table_index)| {
            let table_index = table_index.ok_or(UnavailableMutError::Absent)?;

            if let Some(Some((key, value))) = entries[table_index].take() {
                borrowed[table_index] = Some(idx);
                Ok((&*key, value))
            } else if let Some(idx) = borrowed[table_index] {
                Err(UnavailableMutError::Duplicate(idx))
            } else {
                unreachable!("the entry should be present in entries or an entry in borrowed must be present")
            }
        })
    }

    /// Attempts to get mutable references to `N` values in the map at once.
    ///
    /// Returns an array of length `N` with the results of each query. For soundness,
    /// at most one mutable reference will be returned to any value. An
    /// `Err(UnavailableMutError::Duplicate(i))` in the returned array indicates that a suitable
    /// key-value pair exists, but a mutable reference to the value already occurs at index `i` in
    /// the returned array.
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

    /// Creates an iterator which uses a closure to determine if an element should be removed.
    ///
    /// If the closure returns `true`, the element is removed from the map and yielded.
    /// If the closure returns `false`, or panics, the element remains in the map and will not be
    /// yielded.
    ///
    /// Note that `drain_filter` lets you mutate every value in the filter closure, regardless of
    /// whether you choose to keep or remove it.
    ///
    /// If the iterator is only partially consumed or not consumed at all, each of the remaining
    /// elements will still be subjected to the closure and removed and dropped if it returns true.
    ///
    /// It is unspecified how many more elements will be subjected to the closure
    /// if a panic occurs in the closure, or a panic occurs while dropping an element,
    /// or if the `DrainFilter` value is leaked.
    pub fn drain_filter<F>(&mut self, f: F) -> DrainFilter<'_, K, V, F, N, B>
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        DrainFilter::new(f, self)
    }

    /// Clears the map, returning all key-value pairs as an iterator.
    pub fn drain(&mut self) -> Drain<'_, K, V, N, B> {
        Drain::new(self)
    }
}

impl<K, V, B, const N: usize> ArrayMap<K, V, N, B>
where
    K: Eq + Hash,
    B: BuildHasher,
{
    // Returns an occupied entry if the key is present in the map or None if it is not.
    //
    // This function is more generic than `Self::entry`, because a vacant entry needs to store the key that is passed
    // to the function, but a key with type &Q can not be converted to a key of type K, which is required for the vacant entry!
    fn occupied_entry<Q: ?Sized>(&mut self, key: &Q) -> Option<OccupiedEntry<'_, K, V, N, B>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let index = self.find(&key).occupied()?;

        Some(OccupiedEntry::new(
            &mut self.entries,
            index,
            &self.build_hasher,
            &mut self.len,
        ))
    }

    fn iter_from<Q: ?Sized>(&self, key: &Q) -> IterEntries<'_, K, V, N, B>
    where
        Q: Hash + Eq,
        K: Borrow<Q>,
    {
        IterEntries::new(key, &self.build_hasher, &self.entries)
    }

    fn find<Q: ?Sized>(&self, qkey: &Q) -> FindResult<usize>
    where
        Q: Hash + Eq,
        K: Borrow<Q>,
    {
        for slot in self.iter_from(qkey) {
            if let Slot::Collision { index, key } = slot {
                if key.borrow() == qkey {
                    return FindResult::Occupied(index);
                }
            } else if let Slot::Vacant { index } = slot {
                return FindResult::Vacant(index);
            }
        }

        FindResult::End
    }

    fn occupied_entry_index(&mut self, index: usize) -> Option<OccupiedEntry<'_, K, V, N, B>> {
        debug_assert!(index < self.capacity());

        if self.entries[index].is_none() {
            return None;
        }

        Some(OccupiedEntry::new(
            &mut self.entries,
            index,
            &self.build_hasher,
            &mut self.len,
        ))
    }

    pub(crate) fn remove_entry_index(&mut self, index: usize) -> Option<(K, V)> {
        debug_assert!(index < self.capacity());

        Some(self.occupied_entry_index(index)?.remove_entry())
    }

    pub(crate) fn get_key_value_mut_index(&mut self, index: usize) -> Option<(&K, &mut V)> {
        if index >= N {
            return None;
        }

        match self.entries[index].as_mut() {
            Some((k, v)) => Some((k, v)),
            None => None,
        }
    }
}

/// The error type for [`ArrayMap::get_each_value_mut`] and [`ArrayMap::get_each_key_value_mut`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnavailableMutError {
    /// The requested entry is not present in the table.
    Absent,
    /// The requested entry is present, but a mutable reference to it was already created.
    ///
    /// This includes the index of the mutable reference in the returned array.
    Duplicate(usize),
}

impl<K, V, B: BuildHasher, const N: usize> ArrayMap<K, V, N, B> {
    /// Returns an iterator iterating over the immutable entries of the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// let mut map: ArrayMap<&str, bool, 3> = ArrayMap::new();
    /// map.insert("hello", true)?;
    /// map.insert("valid", true)?;
    /// map.insert("invalid", false)?;
    ///
    /// for (key, value) in map.iter() {
    ///     println!("{}: {}", key, value);
    /// }
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter::new(&self.entries)
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        IterMut::new(&mut self.entries)
    }

    /// An iterator visiting all keys in arbitrary order.
    /// The iterator element type is `&'a K`.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// let mut map: ArrayMap<&str, i32, 11> = ArrayMap::new();
    /// map.insert("a", 1)?;
    /// map.insert("b", 2)?;
    /// map.insert("c", 3)?;
    ///
    /// for key in map.keys() {
    ///     println!("{}", key);
    /// }
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys::new(self.iter())
    }

    /// An iterator visiting all values in arbitrary order.
    /// The iterator element type is `&'a V`.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// let mut map: ArrayMap<&str, i32, 11> = ArrayMap::new();
    /// map.insert("a", 1)?;
    /// map.insert("b", 2)?;
    /// map.insert("c", 3)?;
    ///
    /// for value in map.values() {
    ///     println!("{}", value);
    /// }
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    pub fn values(&self) -> Values<'_, K, V> {
        Values::new(self.iter())
    }

    /// An iterator visiting all values in arbitrary order.
    /// The iterator element type is `&mut V`.
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        ValuesMut::new(self.iter_mut())
    }
}

impl<K, Q: ?Sized, V, B, const N: usize> Index<&Q> for ArrayMap<K, V, N, B>
where
    K: Eq + Hash + Borrow<Q>,
    Q: Eq + Hash,
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

impl<'a, K, V, B: BuildHasher, const N: usize> IntoIterator for &'a ArrayMap<K, V, N, B> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V, B: BuildHasher, const N: usize> IntoIterator for &'a mut ArrayMap<K, V, N, B> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<K, V, B: BuildHasher, const N: usize> IntoIterator for ArrayMap<K, V, N, B> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V, N>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self.entries)
    }
}

impl<K, V, B: BuildHasher + Default, const N: usize> Default for ArrayMap<K, V, N, B> {
    fn default() -> Self {
        Self::with_hasher(B::default())
    }
}

impl<K, V, B: BuildHasher, const N: usize> fmt::Debug for ArrayMap<K, V, N, B>
where
    K: fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<K, V, B, const N: usize> PartialEq<ArrayMap<K, V, N, B>> for ArrayMap<K, V, N, B>
where
    K: Eq + Hash,
    V: PartialEq,
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

impl<K, V, B, const N: usize> Eq for ArrayMap<K, V, N, B>
where
    K: Eq + Hash,
    V: PartialEq,
    B: BuildHasher,
{
}

#[cfg(test)]
mod tests {
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
}
