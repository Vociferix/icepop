//! Lookup and build interface for an [`OrderedMap`] parameterized by [`Portable`].

use super::{Builder, OrderedMap};
use crate::portability::Portable;

use portable::{DefaultHasherSeed, PortableBuildHasher, PortableEq, PortableHash};

impl<K, V> OrderedMap<K, V, DefaultHasherSeed, Portable> {
    /// Creates an empty [`Builder`] with a fresh [`DefaultHasherSeed`].
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::PortableOrderedMap;
    ///
    /// let mut builder = PortableOrderedMap::<&str, u32>::builder();
    /// builder.insert("a", 1);
    ///
    /// assert_eq!(builder.build().get("a"), Some(&1));
    /// ```
    pub fn builder() -> Builder<K, V, DefaultHasherSeed, Portable> {
        Builder::<K, V, DefaultHasherSeed, Portable>::new()
    }

    /// Creates an empty [`Builder`] with room for `capacity` entries.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::PortableOrderedMap;
    ///
    /// let builder = PortableOrderedMap::<&str, u32>::builder_with_capacity(10);
    ///
    /// assert!(builder.capacity() >= 10);
    /// ```
    pub fn builder_with_capacity(capacity: usize) -> Builder<K, V, DefaultHasherSeed, Portable> {
        Builder::<K, V, DefaultHasherSeed, Portable>::with_capacity(capacity)
    }
}

impl<K, V, S> OrderedMap<K, V, S, Portable>
where
    S: PortableBuildHasher,
{
    /// Creates an empty [`Builder`] that will hash with `hasher`.
    ///
    /// The hasher is stored in the finished map and reused for every lookup, so it must produce
    /// the same hash for the same key for as long as the map is read.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{DefaultHasherSeed, PortableOrderedMap};
    ///
    /// let builder = PortableOrderedMap::<&str, u32>::builder_with_hasher(DefaultHasherSeed::with_seed(7));
    ///
    /// assert_eq!(builder.hasher().seed(), 7);
    /// ```
    pub fn builder_with_hasher(hasher: S) -> Builder<K, V, S, Portable> {
        Builder::<K, V, S, Portable>::with_hasher(hasher)
    }

    /// Creates an empty [`Builder`] with room for `capacity` entries, hashing with `hasher`.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{DefaultHasherSeed, PortableOrderedMap};
    ///
    /// let builder = PortableOrderedMap::<&str, u32>::builder_with_capacity_and_hasher(
    ///     10,
    ///     DefaultHasherSeed::with_seed(7),
    /// );
    ///
    /// assert!(builder.capacity() >= 10);
    /// assert_eq!(builder.hasher().seed(), 7);
    /// ```
    pub fn builder_with_capacity_and_hasher(
        capacity: usize,
        hasher: S,
    ) -> Builder<K, V, S, Portable> {
        Builder::<K, V, S, Portable>::with_capacity_and_hasher(capacity, hasher)
    }

    /// Returns the index `key` hashes to, without confirming that it is present.
    ///
    /// # Safety
    ///
    /// The map must not be empty.
    ///
    /// Looking up a key that is not present returns an arbitrary in-bounds index rather than
    /// failing.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::PortableOrderedMap;
    ///
    /// let map: PortableOrderedMap<&str, u32> = [("a", 1u32), ("b", 2)].into_iter().collect();
    ///
    /// // SAFETY: the map is not empty.
    /// let index = unsafe { map.get_index_unchecked("b") };
    ///
    /// assert_eq!(map.index(index), Some((&"b", &2)));
    /// ```
    pub unsafe fn get_index_unchecked<Q>(&self, key: &Q) -> usize
    where
        Q: PortableHash + PortableEq<K> + ?Sized,
    {
        unsafe { self.table.get_index_unchecked(key) }
    }

    /// Returns the index of the entry for `key`, or `None` if there is none.
    ///
    /// The index addresses [`as_slice`](Self::as_slice) and [`index`](Self::index), and stays
    /// valid for the life of the map.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::PortableOrderedMap;
    ///
    /// let map: PortableOrderedMap<&str, u32> = [("a", 1u32), ("b", 2)].into_iter().collect();
    /// let index = map.get_index("b").unwrap();
    ///
    /// assert_eq!(map.index(index), Some((&"b", &2)));
    /// assert_eq!(map.get_index("z"), None);
    /// ```
    pub fn get_index<Q>(&self, key: &Q) -> Option<usize>
    where
        Q: PortableHash + PortableEq<K> + ?Sized,
    {
        self.table.get_index(key)
    }

    /// Returns `true` if the map contains an entry for `key`.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::PortableOrderedMap;
    ///
    /// let map: PortableOrderedMap<&str, u32> = [("a", 1u32)].into_iter().collect();
    ///
    /// assert!(map.contains_key("a"));
    /// assert!(!map.contains_key("z"));
    /// ```
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        Q: PortableHash + PortableEq<K> + ?Sized,
    {
        self.table.contains(key)
    }

    /// Returns the stored key and its value, or `None` if `key` is not present.
    ///
    /// Useful when the stored key carries more than the lookup key does.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::PortableOrderedMap;
    ///
    /// let map: PortableOrderedMap<&str, u32> = [("a", 1u32)].into_iter().collect();
    ///
    /// assert_eq!(map.get_key_value("a"), Some((&"a", &1)));
    /// assert_eq!(map.get_key_value("z"), None);
    /// ```
    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        Q: PortableHash + PortableEq<K> + ?Sized,
    {
        self.table.map_get_key_value(key)
    }

    /// Returns the stored key and a mutable reference to its value.
    ///
    /// The key stays immutable: changing it would invalidate the hash function.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::PortableOrderedMap;
    ///
    /// let mut map: PortableOrderedMap<&str, u32> = [("a", 1u32)].into_iter().collect();
    ///
    /// let (key, value) = map.get_key_value_mut("a").unwrap();
    /// assert_eq!(key, &"a");
    /// *value = 9;
    ///
    /// assert_eq!(map.get("a"), Some(&9));
    /// ```
    pub fn get_key_value_mut<Q>(&mut self, key: &Q) -> Option<(&K, &mut V)>
    where
        Q: PortableHash + PortableEq<K> + ?Sized,
    {
        self.table.map_get_key_value_mut(key)
    }

    /// Returns the value for `key`, or `None` if it is not present.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::PortableOrderedMap;
    ///
    /// let map: PortableOrderedMap<&str, u32> = [("a", 1u32)].into_iter().collect();
    ///
    /// assert_eq!(map.get("a"), Some(&1));
    /// assert_eq!(map.get("z"), None);
    /// ```
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        Q: PortableHash + PortableEq<K> + ?Sized,
    {
        self.table.map_get(key)
    }

    /// Returns a mutable reference to the value for `key`.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::PortableOrderedMap;
    ///
    /// let mut map: PortableOrderedMap<&str, u32> = [("a", 1u32)].into_iter().collect();
    ///
    /// *map.get_mut("a").unwrap() = 9;
    ///
    /// assert_eq!(map.get("a"), Some(&9));
    /// ```
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        Q: PortableHash + PortableEq<K> + ?Sized,
    {
        self.table.map_get_mut(key)
    }

    /// Returns the entry `key` hashes to, without confirming that the key matches.
    ///
    /// # Safety
    ///
    /// The map must not be empty.
    ///
    /// Looking up a key that is not present returns an arbitrary entry rather than failing.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::PortableOrderedMap;
    ///
    /// let map: PortableOrderedMap<&str, u32> = [("a", 1u32)].into_iter().collect();
    ///
    /// // SAFETY: the map is not empty.
    /// assert_eq!(unsafe { map.get_key_value_unchecked("a") }, (&"a", &1));
    /// ```
    pub unsafe fn get_key_value_unchecked<Q>(&self, key: &Q) -> (&K, &V)
    where
        Q: PortableHash + PortableEq<K> + ?Sized,
    {
        unsafe { self.table.map_get_key_value_unchecked(key) }
    }

    /// Returns the entry `key` hashes to with a mutable value, without confirming the key
    /// matches.
    ///
    /// # Safety
    ///
    /// The map must not be empty.
    ///
    /// Looking up a key that is not present returns an arbitrary entry rather than failing.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::PortableOrderedMap;
    ///
    /// let mut map: PortableOrderedMap<&str, u32> = [("a", 1u32)].into_iter().collect();
    ///
    /// // SAFETY: the map is not empty.
    /// *unsafe { map.get_key_value_unchecked_mut("a") }.1 = 9;
    ///
    /// assert_eq!(map.get("a"), Some(&9));
    /// ```
    pub unsafe fn get_key_value_unchecked_mut<Q>(&mut self, key: &Q) -> (&K, &mut V)
    where
        Q: PortableHash + PortableEq<K> + ?Sized,
    {
        unsafe { self.table.map_get_key_value_unchecked_mut(key) }
    }

    /// Returns the value `key` hashes to, without confirming that the key matches.
    ///
    /// # Safety
    ///
    /// The map must not be empty.
    ///
    /// Looking up a key that is not present returns an arbitrary value rather than failing.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::PortableOrderedMap;
    ///
    /// let map: PortableOrderedMap<&str, u32> = [("a", 1u32)].into_iter().collect();
    ///
    /// // SAFETY: the map is not empty.
    /// assert_eq!(unsafe { map.get_unchecked("a") }, &1);
    /// ```
    pub unsafe fn get_unchecked<Q>(&self, key: &Q) -> &V
    where
        Q: PortableHash + PortableEq<K> + ?Sized,
    {
        unsafe { self.table.map_get_unchecked(key) }
    }

    /// Returns a mutable reference to the value `key` hashes to, without confirming the key
    /// matches.
    ///
    /// # Safety
    ///
    /// The map must not be empty.
    ///
    /// Looking up a key that is not present returns an arbitrary value rather than failing.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::PortableOrderedMap;
    ///
    /// let mut map: PortableOrderedMap<&str, u32> = [("a", 1u32)].into_iter().collect();
    ///
    /// // SAFETY: the map is not empty.
    /// *unsafe { map.get_unchecked_mut("a") } = 9;
    ///
    /// assert_eq!(map.get("a"), Some(&9));
    /// ```
    pub unsafe fn get_unchecked_mut<Q>(&mut self, key: &Q) -> &mut V
    where
        Q: PortableHash + PortableEq<K> + ?Sized,
    {
        unsafe { self.table.map_get_unchecked_mut(key) }
    }

    /// Returns `N` entries at once, each with a mutable value.
    ///
    /// Borrowing several values mutably in one call is otherwise impossible, since each
    /// [`get_mut`](Self::get_mut) borrows the whole map. Missing keys yield `None` in place.
    ///
    /// # Panics
    ///
    /// Panics if two keys refer to the same entry, which would alias the same value twice.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::PortableOrderedMap;
    ///
    /// let mut map: PortableOrderedMap<&str, u32> = [("a", 1u32), ("b", 2)].into_iter().collect();
    ///
    /// let [a, b] = map.get_disjoint_key_value_mut(["a", "z"]);
    /// assert_eq!(a.map(|(k, v)| (*k, *v)), Some(("a", 1)));
    /// assert!(b.is_none());
    /// ```
    pub fn get_disjoint_key_value_mut<Q, const N: usize>(
        &mut self,
        keys: [&Q; N],
    ) -> [Option<(&K, &mut V)>; N]
    where
        Q: PortableHash + PortableEq<K> + ?Sized,
    {
        self.table.map_get_disjoint_key_value_mut(keys)
    }

    /// Returns `N` values at once, each mutably borrowed.
    ///
    /// Missing keys yield `None` in place.
    ///
    /// # Panics
    ///
    /// Panics if two keys refer to the same entry, which would alias the same value twice.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::PortableOrderedMap;
    ///
    /// let mut map: PortableOrderedMap<&str, u32> = [("a", 1u32), ("b", 2)].into_iter().collect();
    ///
    /// let [a, b] = map.get_disjoint_mut(["a", "b"]);
    /// *a.unwrap() = 10;
    /// *b.unwrap() = 20;
    ///
    /// assert_eq!(map.get("a"), Some(&10));
    /// assert_eq!(map.get("b"), Some(&20));
    /// ```
    pub fn get_disjoint_mut<Q, const N: usize>(&mut self, keys: [&Q; N]) -> [Option<&mut V>; N]
    where
        Q: PortableHash + PortableEq<K> + ?Sized,
    {
        self.table.map_get_disjoint_mut(keys)
    }

    /// [`get_disjoint_key_value_mut`](Self::get_disjoint_key_value_mut) without the
    /// distinctness check.
    ///
    /// # Safety
    ///
    /// No two keys may refer to the same entry.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::PortableOrderedMap;
    ///
    /// let mut map: PortableOrderedMap<&str, u32> = [("a", 1u32), ("b", 2)].into_iter().collect();
    ///
    /// // SAFETY: "a" and "b" are distinct keys, so they cannot share an entry.
    /// let [a, b] = unsafe { map.get_disjoint_key_value_unchecked_mut(["a", "b"]) };
    /// *a.unwrap().1 = 10;
    /// *b.unwrap().1 = 20;
    ///
    /// assert_eq!(map.get("a"), Some(&10));
    /// ```
    pub unsafe fn get_disjoint_key_value_unchecked_mut<Q, const N: usize>(
        &mut self,
        keys: [&Q; N],
    ) -> [Option<(&K, &mut V)>; N]
    where
        Q: PortableHash + PortableEq<K> + ?Sized,
    {
        unsafe { self.table.map_get_disjoint_key_value_unchecked_mut(keys) }
    }

    /// [`get_disjoint_mut`](Self::get_disjoint_mut) without the distinctness check.
    ///
    /// # Safety
    ///
    /// No two keys may refer to the same entry.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::PortableOrderedMap;
    ///
    /// let mut map: PortableOrderedMap<&str, u32> = [("a", 1u32), ("b", 2)].into_iter().collect();
    ///
    /// // SAFETY: "a" and "b" are distinct keys, so they cannot share an entry.
    /// let [a, b] = unsafe { map.get_disjoint_unchecked_mut(["a", "b"]) };
    /// *a.unwrap() = 10;
    /// *b.unwrap() = 20;
    ///
    /// assert_eq!(map.get("a"), Some(&10));
    /// ```
    pub unsafe fn get_disjoint_unchecked_mut<Q, const N: usize>(
        &mut self,
        keys: [&Q; N],
    ) -> [Option<&mut V>; N]
    where
        Q: PortableHash + PortableEq<K> + ?Sized,
    {
        unsafe { self.table.map_get_disjoint_unchecked_mut(keys) }
    }
}

impl<K, V, S> Default for OrderedMap<K, V, S, Portable>
where
    S: Default,
{
    fn default() -> Self {
        Self {
            table: super::Table::default(),
        }
    }
}

impl<K, V, S> FromIterator<(K, V)> for OrderedMap<K, V, S, Portable>
where
    S: Default + PortableBuildHasher,
    K: PortableHash + PortableEq,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
    {
        Builder::<K, V, S, Portable>::from_iter(iter).build()
    }
}

impl<K, V, S> PartialEq for OrderedMap<K, V, S, Portable>
where
    K: PartialEq,
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<K, V, S> Eq for OrderedMap<K, V, S, Portable>
where
    K: Eq,
    V: Eq,
{
}

impl<K, V> Builder<K, V, DefaultHasherSeed, Portable> {
    /// Creates an empty builder with a fresh [`DefaultHasherSeed`].
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::ordered_map::PortableBuilder;
    ///
    /// let mut builder = PortableBuilder::<&str, u32>::new();
    /// builder.insert("a", 1);
    ///
    /// assert_eq!(builder.len(), 1);
    /// ```
    pub fn new() -> Self {
        Self::with_hasher(DefaultHasherSeed::new())
    }

    /// Creates an empty builder with room for `capacity` entries.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::ordered_map::PortableBuilder;
    ///
    /// let builder = PortableBuilder::<&str, u32>::with_capacity(10);
    ///
    /// assert!(builder.capacity() >= 10);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, DefaultHasherSeed::new())
    }
}

impl<K, V, S> Builder<K, V, S, Portable> {
    /// Creates an empty builder that will hash with `hasher`.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{DefaultHasherSeed, ordered_map::PortableBuilder};
    ///
    /// let builder = PortableBuilder::<&str, u32>::with_hasher(DefaultHasherSeed::with_seed(7));
    ///
    /// assert_eq!(builder.hasher().seed(), 7);
    /// ```
    pub fn with_hasher(hasher: S) -> Self {
        Self {
            builder: super::TableBuilder::with_hasher(hasher),
        }
    }

    /// Creates an empty builder with room for `capacity` entries, hashing with `hasher`.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{DefaultHasherSeed, ordered_map::PortableBuilder};
    ///
    /// let builder =
    ///     PortableBuilder::<&str, u32>::with_capacity_and_hasher(10, DefaultHasherSeed::with_seed(7));
    ///
    /// assert!(builder.capacity() >= 10);
    /// assert_eq!(builder.hasher().seed(), 7);
    /// ```
    pub fn with_capacity_and_hasher(capacity: usize, hasher: S) -> Self {
        Self {
            builder: super::TableBuilder::with_capacity_and_hasher(capacity, hasher),
        }
    }
}

impl<K, V, S> Builder<K, V, S, Portable>
where
    S: PortableBuildHasher,
{
    /// Returns `true` if an entry for `key` has been inserted.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::ordered_map::PortableBuilder;
    ///
    /// let mut builder = PortableBuilder::<&str, u32>::new();
    /// builder.insert("a", 1);
    ///
    /// assert!(builder.contains_key("a"));
    /// assert!(!builder.contains_key("z"));
    /// ```
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        Q: PortableHash + PortableEq<K> + ?Sized,
    {
        self.builder.contains(key)
    }

    /// Returns the inserted key and its value, or `None` if `key` is not present.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::ordered_map::PortableBuilder;
    ///
    /// let mut builder = PortableBuilder::<&str, u32>::new();
    /// builder.insert("a", 1);
    ///
    /// assert_eq!(builder.get_key_value("a"), Some((&"a", &1)));
    /// ```
    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        Q: PortableHash + PortableEq<K> + ?Sized,
    {
        self.builder.map_get_key_value(key)
    }

    /// Returns the inserted key and a mutable reference to its value.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::ordered_map::PortableBuilder;
    ///
    /// let mut builder = PortableBuilder::<&str, u32>::new();
    /// builder.insert("a", 1);
    ///
    /// *builder.get_key_value_mut("a").unwrap().1 = 9;
    ///
    /// assert_eq!(builder.get("a"), Some(&9));
    /// ```
    pub fn get_key_value_mut<Q>(&mut self, key: &Q) -> Option<(&K, &mut V)>
    where
        Q: PortableHash + PortableEq<K> + ?Sized,
    {
        self.builder.map_get_key_value_mut(key)
    }

    /// Returns the value for `key`, or `None` if it is not present.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::ordered_map::PortableBuilder;
    ///
    /// let mut builder = PortableBuilder::<&str, u32>::new();
    /// builder.insert("a", 1);
    ///
    /// assert_eq!(builder.get("a"), Some(&1));
    /// assert_eq!(builder.get("z"), None);
    /// ```
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        Q: PortableHash + PortableEq<K> + ?Sized,
    {
        self.builder.map_get(key)
    }

    /// Returns a mutable reference to the value for `key`.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::ordered_map::PortableBuilder;
    ///
    /// let mut builder = PortableBuilder::<&str, u32>::new();
    /// builder.insert("a", 1);
    ///
    /// *builder.get_mut("a").unwrap() = 9;
    ///
    /// assert_eq!(builder.get("a"), Some(&9));
    /// ```
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        Q: PortableHash + PortableEq<K> + ?Sized,
    {
        self.builder.map_get_mut(key)
    }
}

impl<K, V, S> Builder<K, V, S, Portable>
where
    K: PortableHash + PortableEq,
    S: PortableBuildHasher,
{
    /// Sets the entry for `key` to `update(old)`, where `old` is the value already stored under
    /// an equal key, if any. Returns `true` if the key was not already present.
    ///
    /// This is the way to compute a new value from the old one by value rather than through a
    /// `&mut V`, which matters when `V` is not cheap to modify in place.
    ///
    /// # Panics
    ///
    /// Panics if the builder already holds `u32::MAX` entries. If `update` panics, the entry is
    /// removed from the builder.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::ordered_map::PortableBuilder;
    ///
    /// let mut builder = PortableBuilder::<&str, u32>::new();
    ///
    /// assert!(builder.upsert("a", |old| old.unwrap_or(0) + 1));
    /// assert!(!builder.upsert("a", |old| old.unwrap_or(0) + 1));
    ///
    /// assert_eq!(builder.get("a"), Some(&2));
    /// ```
    pub fn upsert<F>(&mut self, key: K, update: F) -> bool
    where
        F: FnOnce(Option<V>) -> V,
    {
        self.builder.map_upsert(key, update)
    }

    /// Inserts an entry, returning the value it displaced.
    ///
    /// # Panics
    ///
    /// Panics if the builder already holds `u32::MAX` entries.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::ordered_map::PortableBuilder;
    ///
    /// let mut builder = PortableBuilder::<&str, u32>::new();
    ///
    /// assert_eq!(builder.insert("a", 1), None);
    /// assert_eq!(builder.insert("a", 2), Some(1));
    /// assert_eq!(builder.len(), 1);
    /// ```
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.builder.map_insert(key, value)
    }

    /// Returns the value for `key`, inserting `key.to_owned()` mapped to `default()` if there is
    /// none.
    ///
    /// `default` is not called and the key is not cloned when the entry already exists.
    ///
    /// # Panics
    ///
    /// Panics if the builder already holds `u32::MAX` entries.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::ordered_map::PortableBuilder;
    ///
    /// let mut builder = PortableBuilder::<String, u32>::new();
    ///
    /// *builder.get_or_insert_with("a", || 1) += 10;
    ///
    /// assert_eq!(builder.get("a"), Some(&11));
    /// ```
    pub fn get_or_insert_with<Q, F>(&mut self, key: &Q, default: F) -> &mut V
    where
        Q: PortableHash + PortableEq<K> + alloc::borrow::ToOwned<Owned = K> + ?Sized,
        F: FnOnce() -> V,
    {
        self.builder.map_get_or_insert_with(key, default)
    }

    /// Returns the value for `key`, inserting [`V::default()`](Default::default) if there is
    /// none.
    ///
    /// # Panics
    ///
    /// Panics if the builder already holds `u32::MAX` entries.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::ordered_map::PortableBuilder;
    ///
    /// let mut builder = PortableBuilder::<String, u32>::new();
    ///
    /// *builder.get_or_insert_default("a") += 1;
    ///
    /// assert_eq!(builder.get("a"), Some(&1));
    /// ```
    pub fn get_or_insert_default<Q>(&mut self, key: &Q) -> &mut V
    where
        Q: PortableHash + PortableEq<K> + alloc::borrow::ToOwned<Owned = K> + ?Sized,
        V: Default,
    {
        self.builder.map_get_or_insert_default(key)
    }

    /// Returns the value for `key`, inserting `default` if there is none.
    ///
    /// `default` is evaluated whether or not it is needed; use
    /// [`get_or_insert_with`](Self::get_or_insert_with) when producing it is expensive.
    ///
    /// # Panics
    ///
    /// Panics if the builder already holds `u32::MAX` entries.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::ordered_map::PortableBuilder;
    ///
    /// let mut builder = PortableBuilder::<String, u32>::new();
    ///
    /// assert_eq!(builder.get_or_insert("a", 1), &1);
    /// assert_eq!(builder.get_or_insert("a", 2), &1);
    /// ```
    pub fn get_or_insert<Q>(&mut self, key: &Q, default: V) -> &mut V
    where
        Q: PortableHash + PortableEq<K> + alloc::borrow::ToOwned<Owned = K> + ?Sized,
    {
        self.builder.map_get_or_insert(key, default)
    }

    /// Removes the entry for `key` and returns both halves of it.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::ordered_map::PortableBuilder;
    ///
    /// let mut builder = PortableBuilder::<&str, u32>::new();
    /// builder.insert("a", 1);
    ///
    /// assert_eq!(builder.remove_key_value("a"), Some(("a", 1)));
    /// assert_eq!(builder.remove_key_value("a"), None);
    /// ```
    pub fn remove_key_value<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        Q: PortableHash + PortableEq<K> + ?Sized,
    {
        self.builder.take(key)
    }

    /// Removes the entry for `key` and returns its value.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::ordered_map::PortableBuilder;
    ///
    /// let mut builder = PortableBuilder::<&str, u32>::new();
    /// builder.insert("a", 1);
    ///
    /// assert_eq!(builder.remove("a"), Some(1));
    /// assert_eq!(builder.remove("a"), None);
    /// ```
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        Q: PortableHash + PortableEq<K> + ?Sized,
    {
        self.builder.map_remove(key)
    }

    /// Constructs the minimal perfect hash function and freezes the builder into a [`OrderedMap`].
    ///
    /// This is where the cost of the collection is paid; everything before it is an ordinary
    /// hash map.
    ///
    /// # Panics
    ///
    /// Panics if no minimal perfect hash function can be constructed for the inserted keys. That
    /// means the hasher gave two distinct keys the same 64-bit hash under every parameter it was
    /// retried with, or distributed them too poorly to place.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::ordered_map::PortableBuilder;
    ///
    /// let mut builder = PortableBuilder::<&str, u32>::new();
    /// builder.insert("a", 1);
    /// builder.insert("b", 2);
    ///
    /// let map = builder.build();
    ///
    /// assert_eq!(map.get("a"), Some(&1));
    /// ```
    pub fn build(self) -> OrderedMap<K, V, S, Portable> {
        OrderedMap {
            table: self.builder.build(),
        }
    }
}

impl<K, V, S> Default for Builder<K, V, S, Portable>
where
    S: Default,
{
    fn default() -> Self {
        Self::with_hasher(S::default())
    }
}

impl<K, V, S> PartialEq for Builder<K, V, S, Portable>
where
    K: PartialEq,
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.builder.as_slice() == other.builder.as_slice()
    }
}

impl<K, V, S> Eq for Builder<K, V, S, Portable>
where
    K: Eq,
    V: Eq,
{
}

impl<K, V, S> Extend<(K, V)> for Builder<K, V, S, Portable>
where
    S: PortableBuildHasher,
    K: PortableHash + PortableEq,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (K, V)>,
    {
        let iter = iter.into_iter();
        let cap = iter.size_hint().0;
        self.reserve(cap);
        iter.for_each(|(k, v)| {
            self.insert(k, v);
        });
    }
}

impl<K, V, S> FromIterator<(K, V)> for Builder<K, V, S, Portable>
where
    S: Default + PortableBuildHasher,
    K: PortableHash + PortableEq,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
    {
        let iter = iter.into_iter();
        let cap = iter.size_hint().0;
        let mut builder = Self::with_capacity_and_hasher(cap, S::default());
        iter.for_each(|(k, v)| {
            builder.insert(k, v);
        });
        builder
    }
}

cfg_select!(feature = "serde" => {
    impl<K, V, S> serde::Serialize for OrderedMap<K, V, S, Portable>
    where
        K: serde::Serialize,
        V: serde::Serialize,
        S: serde::Serialize,
    {
        fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
        where
            Ser: serde::Serializer,
        {
            self.table.serialize(serializer)
        }
    }

    impl<'de, K, V, S> serde::Deserialize<'de> for OrderedMap<K, V, S, Portable>
    where
        K: serde::Deserialize<'de>,
        V: serde::Deserialize<'de>,
        S: serde::Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            super::Table::deserialize(deserializer).map(|table| Self { table })
        }
    }
} _ => {});
