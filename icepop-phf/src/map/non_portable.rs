//! Lookup and build interface for a [`Map`] parameterized by [`NonPortable`].

use super::{Builder, Map};
use crate::portability::NonPortable;

use equivalent::Equivalent;
use portable::DefaultHasherSeed;

use core::hash::{BuildHasher, Hash};

impl<K, V> Map<K, V, DefaultHasherSeed, NonPortable> {
    /// Creates an empty [`Builder`] with a fresh [`DefaultHasherSeed`].
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Map;
    ///
    /// let mut builder = Map::<&str, u32>::builder();
    /// builder.insert("a", 1);
    ///
    /// assert_eq!(builder.build().get("a"), Some(&1));
    /// ```
    pub fn builder() -> Builder<K, V, DefaultHasherSeed, NonPortable> {
        Builder::<K, V, DefaultHasherSeed, NonPortable>::new()
    }

    /// Creates an empty [`Builder`] with room for `capacity` entries.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Map;
    ///
    /// let builder = Map::<&str, u32>::builder_with_capacity(10);
    ///
    /// assert!(builder.capacity() >= 10);
    /// ```
    pub fn builder_with_capacity(capacity: usize) -> Builder<K, V, DefaultHasherSeed, NonPortable> {
        Builder::<K, V, DefaultHasherSeed, NonPortable>::with_capacity(capacity)
    }
}

impl<K, V, S> Map<K, V, S, NonPortable>
where
    S: BuildHasher,
{
    /// Creates an empty [`Builder`] that will hash with `hasher`.
    ///
    /// The hasher is stored in the finished map and reused for every lookup, so it must produce
    /// the same hash for the same key for as long as the map is read.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{DefaultHasherSeed, Map};
    ///
    /// let builder = Map::<&str, u32>::builder_with_hasher(DefaultHasherSeed::with_seed(7));
    ///
    /// assert_eq!(builder.hasher().seed(), 7);
    /// ```
    pub fn builder_with_hasher(hasher: S) -> Builder<K, V, S, NonPortable> {
        Builder::<K, V, S, NonPortable>::with_hasher(hasher)
    }

    /// Creates an empty [`Builder`] with room for `capacity` entries, hashing with `hasher`.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{DefaultHasherSeed, Map};
    ///
    /// let builder = Map::<&str, u32>::builder_with_capacity_and_hasher(
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
    ) -> Builder<K, V, S, NonPortable> {
        Builder::<K, V, S, NonPortable>::with_capacity_and_hasher(capacity, hasher)
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
    /// use icepop_phf::Map;
    ///
    /// let map: Map<&str, u32> = [("a", 1u32), ("b", 2)].into_iter().collect();
    ///
    /// // SAFETY: the map is not empty.
    /// let index = unsafe { map.get_index_unchecked("b") };
    ///
    /// assert_eq!(map.index(index), Some((&"b", &2)));
    /// ```
    pub unsafe fn get_index_unchecked<Q>(&self, key: &Q) -> usize
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        // SAFETY: `Table::get_index_unchecked` has this function's contract, which the caller
        // upheld.
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
    /// use icepop_phf::Map;
    ///
    /// let map: Map<&str, u32> = [("a", 1u32), ("b", 2)].into_iter().collect();
    /// let index = map.get_index("b").unwrap();
    ///
    /// assert_eq!(map.index(index), Some((&"b", &2)));
    /// assert_eq!(map.get_index("z"), None);
    /// ```
    pub fn get_index<Q>(&self, key: &Q) -> Option<usize>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.table.get_index(key)
    }

    /// Returns `true` if the map contains an entry for `key`.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Map;
    ///
    /// let map: Map<&str, u32> = [("a", 1u32)].into_iter().collect();
    ///
    /// assert!(map.contains_key("a"));
    /// assert!(!map.contains_key("z"));
    /// ```
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        Q: Hash + Equivalent<K> + ?Sized,
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
    /// use icepop_phf::Map;
    ///
    /// let map: Map<&str, u32> = [("a", 1u32)].into_iter().collect();
    ///
    /// assert_eq!(map.get_key_value("a"), Some((&"a", &1)));
    /// assert_eq!(map.get_key_value("z"), None);
    /// ```
    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        Q: Hash + Equivalent<K> + ?Sized,
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
    /// use icepop_phf::Map;
    ///
    /// let mut map: Map<&str, u32> = [("a", 1u32)].into_iter().collect();
    ///
    /// let (key, value) = map.get_key_value_mut("a").unwrap();
    /// assert_eq!(key, &"a");
    /// *value = 9;
    ///
    /// assert_eq!(map.get("a"), Some(&9));
    /// ```
    pub fn get_key_value_mut<Q>(&mut self, key: &Q) -> Option<(&K, &mut V)>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.table.map_get_key_value_mut(key)
    }

    /// Returns the value for `key`, or `None` if it is not present.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Map;
    ///
    /// let map: Map<&str, u32> = [("a", 1u32)].into_iter().collect();
    ///
    /// assert_eq!(map.get("a"), Some(&1));
    /// assert_eq!(map.get("z"), None);
    /// ```
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.table.map_get(key)
    }

    /// Returns a mutable reference to the value for `key`.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Map;
    ///
    /// let mut map: Map<&str, u32> = [("a", 1u32)].into_iter().collect();
    ///
    /// *map.get_mut("a").unwrap() = 9;
    ///
    /// assert_eq!(map.get("a"), Some(&9));
    /// ```
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        Q: Hash + Equivalent<K> + ?Sized,
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
    /// use icepop_phf::Map;
    ///
    /// let map: Map<&str, u32> = [("a", 1u32)].into_iter().collect();
    ///
    /// // SAFETY: the map is not empty.
    /// assert_eq!(unsafe { map.get_key_value_unchecked("a") }, (&"a", &1));
    /// ```
    pub unsafe fn get_key_value_unchecked<Q>(&self, key: &Q) -> (&K, &V)
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        // SAFETY: `Table::map_get_key_value_unchecked` has this function's contract, which the
        // caller upheld.
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
    /// use icepop_phf::Map;
    ///
    /// let mut map: Map<&str, u32> = [("a", 1u32)].into_iter().collect();
    ///
    /// // SAFETY: the map is not empty.
    /// *unsafe { map.get_key_value_unchecked_mut("a") }.1 = 9;
    ///
    /// assert_eq!(map.get("a"), Some(&9));
    /// ```
    pub unsafe fn get_key_value_unchecked_mut<Q>(&mut self, key: &Q) -> (&K, &mut V)
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        // SAFETY: `Table::map_get_key_value_unchecked_mut` has this function's contract, which the
        // caller upheld.
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
    /// use icepop_phf::Map;
    ///
    /// let map: Map<&str, u32> = [("a", 1u32)].into_iter().collect();
    ///
    /// // SAFETY: the map is not empty.
    /// assert_eq!(unsafe { map.get_unchecked("a") }, &1);
    /// ```
    pub unsafe fn get_unchecked<Q>(&self, key: &Q) -> &V
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        // SAFETY: `Table::map_get_unchecked` has this function's contract, which the caller upheld.
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
    /// use icepop_phf::Map;
    ///
    /// let mut map: Map<&str, u32> = [("a", 1u32)].into_iter().collect();
    ///
    /// // SAFETY: the map is not empty.
    /// *unsafe { map.get_unchecked_mut("a") } = 9;
    ///
    /// assert_eq!(map.get("a"), Some(&9));
    /// ```
    pub unsafe fn get_unchecked_mut<Q>(&mut self, key: &Q) -> &mut V
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        // SAFETY: `Table::map_get_unchecked_mut` has this function's contract, which the caller
        // upheld.
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
    /// use icepop_phf::Map;
    ///
    /// let mut map: Map<&str, u32> = [("a", 1u32), ("b", 2)].into_iter().collect();
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
        Q: Hash + Equivalent<K> + ?Sized,
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
    /// use icepop_phf::Map;
    ///
    /// let mut map: Map<&str, u32> = [("a", 1u32), ("b", 2)].into_iter().collect();
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
        Q: Hash + Equivalent<K> + ?Sized,
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
    /// use icepop_phf::Map;
    ///
    /// let mut map: Map<&str, u32> = [("a", 1u32), ("b", 2)].into_iter().collect();
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
        Q: Hash + Equivalent<K> + ?Sized,
    {
        // SAFETY: `Table::map_get_disjoint_key_value_unchecked_mut` has this function's contract,
        // which the caller upheld.
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
    /// use icepop_phf::Map;
    ///
    /// let mut map: Map<&str, u32> = [("a", 1u32), ("b", 2)].into_iter().collect();
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
        Q: Hash + Equivalent<K> + ?Sized,
    {
        // SAFETY: `Table::map_get_disjoint_unchecked_mut` has this function's contract, which the
        // caller upheld.
        unsafe { self.table.map_get_disjoint_unchecked_mut(keys) }
    }
}

impl<K, V, S> Default for Map<K, V, S, NonPortable>
where
    S: Default,
{
    fn default() -> Self {
        Self {
            table: super::Table::default(),
        }
    }
}

impl<K, V, S> FromIterator<(K, V)> for Map<K, V, S, NonPortable>
where
    S: Default + BuildHasher,
    K: Hash + Eq,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
    {
        Builder::<K, V, S, NonPortable>::from_iter(iter).build()
    }
}

impl<K, V, S> PartialEq for Map<K, V, S, NonPortable>
where
    K: Hash + Eq,
    V: PartialEq,
    S: BuildHasher,
{
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter()
            .all(|(key, value)| other.get(key).is_some_and(|other| value == other))
    }
}

impl<K, V, S> Eq for Map<K, V, S, NonPortable>
where
    K: Hash + Eq,
    V: Eq,
    S: BuildHasher,
{
}

impl<K, V> Builder<K, V, DefaultHasherSeed, NonPortable> {
    /// Creates an empty builder with a fresh [`DefaultHasherSeed`].
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::map::Builder;
    ///
    /// let mut builder = Builder::<&str, u32>::new();
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
    /// use icepop_phf::map::Builder;
    ///
    /// let builder = Builder::<&str, u32>::with_capacity(10);
    ///
    /// assert!(builder.capacity() >= 10);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, DefaultHasherSeed::new())
    }
}

impl<K, V, S> Builder<K, V, S, NonPortable> {
    /// Creates an empty builder that will hash with `hasher`.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{DefaultHasherSeed, map::Builder};
    ///
    /// let builder = Builder::<&str, u32>::with_hasher(DefaultHasherSeed::with_seed(7));
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
    /// use icepop_phf::{DefaultHasherSeed, map::Builder};
    ///
    /// let builder =
    ///     Builder::<&str, u32>::with_capacity_and_hasher(10, DefaultHasherSeed::with_seed(7));
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

impl<K, V, S> Builder<K, V, S, NonPortable>
where
    S: BuildHasher,
{
    /// Returns `true` if an entry for `key` has been inserted.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::map::Builder;
    ///
    /// let mut builder = Builder::<&str, u32>::new();
    /// builder.insert("a", 1);
    ///
    /// assert!(builder.contains_key("a"));
    /// assert!(!builder.contains_key("z"));
    /// ```
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.builder.contains(key)
    }

    /// Returns the inserted key and its value, or `None` if `key` is not present.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::map::Builder;
    ///
    /// let mut builder = Builder::<&str, u32>::new();
    /// builder.insert("a", 1);
    ///
    /// assert_eq!(builder.get_key_value("a"), Some((&"a", &1)));
    /// ```
    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.builder.map_get_key_value(key)
    }

    /// Returns the inserted key and a mutable reference to its value.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::map::Builder;
    ///
    /// let mut builder = Builder::<&str, u32>::new();
    /// builder.insert("a", 1);
    ///
    /// *builder.get_key_value_mut("a").unwrap().1 = 9;
    ///
    /// assert_eq!(builder.get("a"), Some(&9));
    /// ```
    pub fn get_key_value_mut<Q>(&mut self, key: &Q) -> Option<(&K, &mut V)>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.builder.map_get_key_value_mut(key)
    }

    /// Returns the value for `key`, or `None` if it is not present.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::map::Builder;
    ///
    /// let mut builder = Builder::<&str, u32>::new();
    /// builder.insert("a", 1);
    ///
    /// assert_eq!(builder.get("a"), Some(&1));
    /// assert_eq!(builder.get("z"), None);
    /// ```
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.builder.map_get(key)
    }

    /// Returns a mutable reference to the value for `key`.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::map::Builder;
    ///
    /// let mut builder = Builder::<&str, u32>::new();
    /// builder.insert("a", 1);
    ///
    /// *builder.get_mut("a").unwrap() = 9;
    ///
    /// assert_eq!(builder.get("a"), Some(&9));
    /// ```
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.builder.map_get_mut(key)
    }
}

impl<K, V, S> Builder<K, V, S, NonPortable>
where
    K: Hash + Eq,
    S: BuildHasher,
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
    /// use icepop_phf::map::Builder;
    ///
    /// let mut builder = Builder::<&str, u32>::new();
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
    /// use icepop_phf::map::Builder;
    ///
    /// let mut builder = Builder::<&str, u32>::new();
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
    /// use icepop_phf::map::Builder;
    ///
    /// let mut builder = Builder::<String, u32>::new();
    ///
    /// *builder.get_or_insert_with("a", || 1) += 10;
    ///
    /// assert_eq!(builder.get("a"), Some(&11));
    /// ```
    pub fn get_or_insert_with<Q, F>(&mut self, key: &Q, default: F) -> &mut V
    where
        Q: Hash + Equivalent<K> + alloc::borrow::ToOwned<Owned = K> + ?Sized,
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
    /// use icepop_phf::map::Builder;
    ///
    /// let mut builder = Builder::<String, u32>::new();
    ///
    /// *builder.get_or_insert_default("a") += 1;
    ///
    /// assert_eq!(builder.get("a"), Some(&1));
    /// ```
    pub fn get_or_insert_default<Q>(&mut self, key: &Q) -> &mut V
    where
        Q: Hash + Equivalent<K> + alloc::borrow::ToOwned<Owned = K> + ?Sized,
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
    /// use icepop_phf::map::Builder;
    ///
    /// let mut builder = Builder::<String, u32>::new();
    ///
    /// assert_eq!(builder.get_or_insert("a", 1), &1);
    /// assert_eq!(builder.get_or_insert("a", 2), &1);
    /// ```
    pub fn get_or_insert<Q>(&mut self, key: &Q, default: V) -> &mut V
    where
        Q: Hash + Equivalent<K> + alloc::borrow::ToOwned<Owned = K> + ?Sized,
    {
        self.builder.map_get_or_insert(key, default)
    }

    /// Removes the entry for `key` and returns both halves of it.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::map::Builder;
    ///
    /// let mut builder = Builder::<&str, u32>::new();
    /// builder.insert("a", 1);
    ///
    /// assert_eq!(builder.remove_key_value("a"), Some(("a", 1)));
    /// assert_eq!(builder.remove_key_value("a"), None);
    /// ```
    pub fn remove_key_value<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.builder.take(key)
    }

    /// Removes the entry for `key` and returns its value.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::map::Builder;
    ///
    /// let mut builder = Builder::<&str, u32>::new();
    /// builder.insert("a", 1);
    ///
    /// assert_eq!(builder.remove("a"), Some(1));
    /// assert_eq!(builder.remove("a"), None);
    /// ```
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.builder.map_remove(key)
    }

    /// Constructs the minimal perfect hash function and freezes the builder into a [`Map`].
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
    /// use icepop_phf::map::Builder;
    ///
    /// let mut builder = Builder::<&str, u32>::new();
    /// builder.insert("a", 1);
    /// builder.insert("b", 2);
    ///
    /// let map = builder.build();
    ///
    /// assert_eq!(map.get("a"), Some(&1));
    /// ```
    pub fn build(self) -> Map<K, V, S, NonPortable> {
        Map {
            table: self.builder.build(),
        }
    }
}

impl<K, V, S> Default for Builder<K, V, S, NonPortable>
where
    S: Default,
{
    fn default() -> Self {
        Self::with_hasher(S::default())
    }
}

impl<K, V, S> PartialEq for Builder<K, V, S, NonPortable>
where
    K: Hash + Eq,
    V: PartialEq,
    S: BuildHasher,
{
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter()
            .all(|(key, value)| other.get(key).is_some_and(|other| value == other))
    }
}

impl<K, V, S> Eq for Builder<K, V, S, NonPortable>
where
    K: Hash + Eq,
    V: Eq,
    S: BuildHasher,
{
}

impl<K, V, S> Extend<(K, V)> for Builder<K, V, S, NonPortable>
where
    S: BuildHasher,
    K: Hash + Eq,
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

impl<K, V, S> FromIterator<(K, V)> for Builder<K, V, S, NonPortable>
where
    S: Default + BuildHasher,
    K: Hash + Eq,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Map;

    use alloc::vec::Vec;

    fn map_of(entries: impl IntoIterator<Item = (u32, u32)>) -> Map<u32, u32> {
        let mut builder = Map::<u32, u32>::builder_with_hasher(DefaultHasherSeed::with_seed(1));
        for (k, v) in entries {
            builder.insert(k, v);
        }
        builder.build()
    }

    #[test]
    fn every_constructor_reaches_the_same_builder() {
        assert!(Map::<u32, u32>::builder().build().is_empty());
        assert!(Map::<u32, u32>::builder_with_capacity(8).capacity() >= 8);
        assert_eq!(
            Map::<u32, u32>::builder_with_hasher(DefaultHasherSeed::with_seed(3))
                .hasher()
                .seed(),
            3,
        );
        assert!(
            Map::<u32, u32>::builder_with_capacity_and_hasher(8, DefaultHasherSeed::with_seed(3))
                .capacity()
                >= 8
        );

        assert!(Builder::<u32, u32>::new().build().is_empty());
        assert!(Builder::<u32, u32>::with_capacity(8).capacity() >= 8);
        assert_eq!(
            Builder::<u32, u32>::with_hasher(DefaultHasherSeed::with_seed(3))
                .hasher()
                .seed(),
            3,
        );
        assert!(
            Builder::<u32, u32>::with_capacity_and_hasher(8, DefaultHasherSeed::with_seed(3))
                .capacity()
                >= 8
        );

        assert!(Map::<u32, u32>::default().is_empty());
        assert!(Builder::<u32, u32>::default().build().is_empty());
    }

    #[test]
    fn lookups_reach_keys_values_and_absent_entries() {
        let mut map = map_of((0..8).map(|k| (k, k * 10)));

        for k in 0..8u32 {
            let index = map.get_index(&k).unwrap();
            assert!(map.contains_key(&k));
            assert_eq!(map.get(&k), Some(&(k * 10)));
            assert_eq!(map.get_key_value(&k), Some((&k, &(k * 10))));

            // SAFETY: the map is not empty.
            unsafe {
                assert_eq!(map.get_index_unchecked(&k), index);
                assert_eq!(map.get_unchecked(&k), &(k * 10));
                assert_eq!(map.get_key_value_unchecked(&k), (&k, &(k * 10)));
            }
        }

        assert_eq!(map.get_index(&99u32), None);
        assert!(!map.contains_key(&99u32));
        assert_eq!(map.get(&99u32), None);
        assert_eq!(map.get_key_value(&99u32), None);
        assert_eq!(map.get_mut(&99u32), None);
        assert_eq!(map.get_key_value_mut(&99u32), None);

        *map.get_mut(&1u32).unwrap() = 1;
        *map.get_key_value_mut(&1u32).unwrap().1 += 1;
        // SAFETY: the map is not empty.
        unsafe {
            *map.get_unchecked_mut(&1u32) += 1;
            *map.get_key_value_unchecked_mut(&1u32).1 += 1;
        }
        assert_eq!(map.get(&1u32), Some(&4));
    }

    #[test]
    fn disjoint_accessors_borrow_several_values_at_once() {
        let mut map = map_of((0..8).map(|k| (k, k * 10)));

        let [a, b, missing] = map.get_disjoint_mut([&1u32, &2u32, &99u32]);
        *a.unwrap() = 111;
        *b.unwrap() = 222;
        assert!(missing.is_none());

        let [c, missing] = map.get_disjoint_key_value_mut([&3u32, &99u32]);
        let (key, value) = c.unwrap();
        assert_eq!(key, &3);
        *value = 333;
        assert!(missing.is_none());

        // SAFETY: the keys are pairwise distinct, so they cannot share an entry.
        let [d, e] = unsafe { map.get_disjoint_unchecked_mut([&4u32, &5u32]) };
        *d.unwrap() = 444;
        *e.unwrap() = 555;

        // SAFETY: the keys are pairwise distinct, so they cannot share an entry.
        let [f, g] = unsafe { map.get_disjoint_key_value_unchecked_mut([&6u32, &7u32]) };
        *f.unwrap().1 = 666;
        *g.unwrap().1 = 777;

        for k in 1..8u32 {
            assert_eq!(map.get(&k), Some(&(k * 111)));
        }
    }

    #[test]
    fn the_builder_inserts_updates_and_removes() {
        let mut builder = Map::<u32, u32>::builder();

        assert_eq!(builder.insert(1, 10), None);
        assert_eq!(builder.insert(1, 11), Some(10));
        assert!(builder.upsert(2, |old| {
            assert_eq!(old, None);
            20
        }));
        assert!(!builder.upsert(2, |old| old.unwrap() + 1));

        assert!(builder.contains_key(&1u32));
        assert_eq!(builder.get(&2u32), Some(&21));
        assert_eq!(builder.get_key_value(&2u32), Some((&2, &21)));
        assert_eq!(builder.get(&99u32), None);
        assert!(!builder.contains_key(&99u32));
        assert_eq!(builder.get_key_value(&99u32), None);

        *builder.get_mut(&1u32).unwrap() = 12;
        *builder.get_key_value_mut(&1u32).unwrap().1 += 1;
        assert_eq!(builder.get(&1u32), Some(&13));

        assert_eq!(*builder.get_or_insert_with(&3u32, || 30), 30);
        assert_eq!(*builder.get_or_insert(&4u32, 40), 40);
        assert_eq!(*builder.get_or_insert_default(&5u32), 0);
        assert_eq!(*builder.get_or_insert(&4u32, 99), 40);

        assert_eq!(builder.remove(&5u32), Some(0));
        assert_eq!(builder.remove(&5u32), None);
        assert_eq!(builder.remove_key_value(&4u32), Some((4, 40)));
        assert_eq!(builder.remove_key_value(&4u32), None);
        assert_eq!(builder.len(), 3);
    }

    #[test]
    fn collecting_keeps_the_last_of_two_equal_keys() {
        // `insert` semantics, matching the standard library's `HashMap`.
        let map: Map<u32, u32> = [(1u32, 10u32), (2, 20), (1, 11)].into_iter().collect();
        assert_eq!(map.get(&1u32), Some(&11));
        assert_eq!(map.len(), 2);

        let mut builder: Builder<u32, u32> = [(1u32, 10u32)].into_iter().collect();
        builder.extend([(1u32, 99u32), (2, 20)]);
        assert_eq!(builder.get(&1u32), Some(&99));
        assert_eq!(builder.len(), 2);
    }

    #[test]
    fn equality_compares_keys_and_values() {
        let a = map_of((0..8).map(|k| (k, k * 10)));

        assert_eq!(a, map_of((0..8).rev().map(|k| (k, k * 10))));
        assert_ne!(a, map_of((0..7).map(|k| (k, k * 10))));
        assert_ne!(
            a,
            map_of((0..8).map(|k| (k, if k == 3 { 99 } else { k * 10 })))
        );

        let mut ba: Builder<u32, u32> = [(1u32, 10u32)].into_iter().collect();
        let bb: Builder<u32, u32> = [(1u32, 10u32)].into_iter().collect();
        assert_eq!(ba, bb);
        ba.insert(1, 11);
        assert_ne!(ba, bb);

        let _ = a.values().collect::<Vec<_>>();
    }
}
