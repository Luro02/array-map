use core::fmt;
use core::hash::{BuildHasher, Hash};

use crate::map::occupied::OccupiedEntry;
use crate::map::vacant::VacantEntry;
use crate::raw::RawTable;

/// A view into a single entry in a map, which may either be vacant or occupied.
///
/// This `enum` is constructed from the [`entry`] method on [`ArrayMap`].
///
/// [`ArrayMap`]: crate::ArrayMap
/// [`entry`]: crate::ArrayMap::entry
pub enum Entry<'a, K: 'a, V: 'a, R: RawTable<(K, V)>, B: BuildHasher> {
    /// An occupied entry.
    Occupied(OccupiedEntry<'a, K, V, R, B>),
    /// A vacant entry.
    Vacant(VacantEntry<'a, K, V, R, B>),
}

impl<'a, K, V, R: RawTable<(K, V)>, B: BuildHasher> Entry<'a, K, V, R, B> {
    /// Returns a reference to this entry's key.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// let mut map: ArrayMap<&str, u32, 19> = ArrayMap::new();
    /// assert_eq!(map.entry("poneyland")?.key(), &"poneyland");
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    #[must_use]
    pub fn key(&self) -> &K {
        match &self {
            Self::Occupied(entry) => entry.key(),
            Self::Vacant(entry) => entry.key(),
        }
    }

    /// Provides in-place mutable access to an occupied entry before any
    /// potential inserts into the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// let mut map: ArrayMap<&str, u32, 17> = ArrayMap::new();
    ///
    /// map.entry("poneyland")?
    ///     .and_modify(|e| *e += 1)
    ///     .or_insert(42);
    /// assert_eq!(map["poneyland"], 42);
    ///
    /// map.entry("poneyland")?
    ///     .and_modify(|e| *e += 1)
    ///     .or_insert(42);
    /// assert_eq!(map["poneyland"], 43);
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    #[must_use]
    pub fn and_modify<F: FnOnce(&mut V)>(self, f: F) -> Self {
        match self {
            Self::Occupied(mut entry) => {
                f(entry.get_mut());
                Self::Occupied(entry)
            }
            Self::Vacant(entry) => Self::Vacant(entry),
        }
    }
}

impl<'a, K: Hash, V, R: RawTable<(K, V)>, B: BuildHasher> Entry<'a, K, V, R, B> {
    /// Sets the value of the entry, and returns an [`OccupiedEntry`].
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// let mut map: ArrayMap<&str, u32, 19> = ArrayMap::new();
    /// let entry = map.entry("horseyland")?.insert_entry(42);
    ///
    /// assert_eq!(entry.key(), &"horseyland");
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    pub fn insert_entry(self, value: V) -> OccupiedEntry<'a, K, V, R, B> {
        match self {
            Self::Occupied(mut entry) => {
                entry.insert(value);
                entry
            }
            Self::Vacant(entry) => entry.insert_entry(value),
        }
    }

    /// Sets the value of the entry, and returns the old value if the entry was
    /// occupied.
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
    /// castles.try_insert("Anvil Castle", Status::Occupied);
    ///
    /// let castle_entry = castles.entry("Anvil Castle")?;
    /// assert_eq!(castle_entry.insert(Status::Empty), Some(Status::Occupied));
    /// assert_eq!(castles["Anvil Castle"], Status::Empty);
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    pub fn insert(self, value: V) -> Option<V> {
        match self {
            Self::Occupied(mut entry) => Some(entry.insert(value)),
            Self::Vacant(entry) => {
                entry.insert(value);
                None
            }
        }
    }

    /// Ensures a value is in the entry by inserting the default if empty,
    /// and returns a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// let mut map: ArrayMap<&str, u32, 19> = ArrayMap::new();
    /// map.entry("poneyland")?.or_insert(42);
    /// assert_eq!(map["poneyland"], 42);
    ///
    /// *map.entry("poneyland")?.or_insert(10) *= 2;
    /// assert_eq!(map["poneyland"], 84);
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    pub fn or_insert(self, default: V) -> &'a mut V {
        Self::or_insert_with(self, || default)
    }

    /// Ensures a value is in the entry by inserting the result of the default
    /// function if empty, and returns a mutable reference to the value in
    /// the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// let mut map: ArrayMap<&str, String, 17> = ArrayMap::new();
    /// let s = "hoho".to_string();
    ///
    /// map.entry("poneyland")?.or_insert_with(|| s);
    ///
    /// assert_eq!(map["poneyland"], "hoho".to_string());
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        match self {
            Self::Occupied(o) => o.into_mut(),
            Self::Vacant(v) => v.insert(default()),
        }
    }

    /// Ensures a value is in the entry by inserting, if empty, the result of
    /// the default function. This method allows for generating key-derived
    /// values for insertion by providing the default function a reference
    /// to the key that was moved during the `.entry(key)` method call.
    ///
    /// The reference to the moved key is provided so that cloning or copying
    /// the key is unnecessary, unlike with `.or_insert_with(|| ... )`.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// let mut map: ArrayMap<&str, usize, 17> = ArrayMap::new();
    ///
    /// map.entry("poneyland")?
    ///     .or_insert_with_key(|key| key.chars().count());
    ///
    /// assert_eq!(map["poneyland"], 9);
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    pub fn or_insert_with_key<F: FnOnce(&K) -> V>(self, default: F) -> &'a mut V {
        match self {
            Self::Occupied(entry) => entry.into_mut(),
            Self::Vacant(entry) => {
                let value = default(entry.key());
                entry.insert(value)
            }
        }
    }
}

impl<'a, K: Hash, V: Default, R: RawTable<(K, V)>, B: BuildHasher> Entry<'a, K, V, R, B> {
    /// Ensures a value is in the entry by inserting the default value if empty,
    /// and returns a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// let mut map: ArrayMap<&str, Option<u32>, 19> = ArrayMap::new();
    /// map.entry("poneyland")?.or_default();
    ///
    /// assert_eq!(map["poneyland"], None);
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    pub fn or_default(self) -> &'a mut V
    where
        V: Default,
    {
        Self::or_insert(self, V::default())
    }
}

impl<'a, K: Hash + Eq, V, R: RawTable<(K, V)>, B: BuildHasher> Entry<'a, K, V, R, B> {
    /// Ensures that no value is associated with the key and returns a
    /// `VacantEntry`.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// let mut map: ArrayMap<&str, &str, 11> = ArrayMap::new();
    ///
    /// let vacant_entry = map.entry("good")?.remove_entry();
    ///
    /// let key = vacant_entry.into_key();
    /// assert_eq!(key, "good");
    /// assert_eq!(map.contains_key(&key), false);
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    #[must_use = "directly call ArrayMap::remove if you dont use the VacantEntry"]
    pub fn remove_entry(self) -> VacantEntry<'a, K, V, R, B> {
        match self {
            Self::Occupied(entry) => entry.remove_entry_helper().0,
            Self::Vacant(entry) => entry,
        }
    }
}

impl<'a, K, V, R, B> From<OccupiedEntry<'a, K, V, R, B>> for Entry<'a, K, V, R, B>
where
    B: BuildHasher,
    R: RawTable<(K, V)>,
{
    fn from(value: OccupiedEntry<'a, K, V, R, B>) -> Self {
        Self::Occupied(value)
    }
}

impl<'a, K, V, R, B> From<VacantEntry<'a, K, V, R, B>> for Entry<'a, K, V, R, B>
where
    B: BuildHasher,
    R: RawTable<(K, V)>,
{
    fn from(value: VacantEntry<'a, K, V, R, B>) -> Self {
        Self::Vacant(value)
    }
}

impl<'a, K, V, R, B> fmt::Debug for Entry<'a, K, V, R, B>
where
    K: fmt::Debug,
    V: fmt::Debug,
    B: BuildHasher,
    R: RawTable<(K, V)>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Vacant(value) => f.debug_tuple(stringify!(Entry)).field(value).finish(),
            Self::Occupied(value) => f.debug_tuple(stringify!(Entry)).field(value).finish(),
        }
    }
}
