//! A minimal perfect hash map whose entries are stored in an arbitrary order.

use crate::portability::{MapOps, NonPortable, Portable};
use crate::table::{Builder as TableBuilder, Table};

use ::portable::DefaultHasherSeed;

#[doc(inline)]
pub use crate::map_iters::{
    IntoIter, IntoKeys, IntoValues, Iter, IterMut, Keys, Values, ValuesMut,
};

#[cfg(feature = "rkyv")]
pub mod rkyv;

mod non_portable;
mod portable;

/// A minimal perfect hash map: built once, then read-only.
///
/// A lookup examines exactly one entry and the table stores no empty slots, but the whole key
/// set must be known up front. Build one by filling a [`Builder`] and calling `build`.
///
/// Frozen refers to the key set: values remain mutable in place, since changing a value
/// cannot disturb the hash function. Keys are handed out by shared reference only.
///
/// Entries are permuted at build time so that a key's hash slot is its index, which is what
/// lets a lookup avoid an indirection. Iteration and [`as_slice`](Self::as_slice) therefore
/// visit an arbitrary order; use [`OrderedMap`](crate::OrderedMap) to keep the order entries
/// were inserted.
///
/// `P` selects the lookup interface and is either [`NonPortable`], the default, or
/// [`Portable`]; see the [crate documentation](crate#portability). [`PortableMap`] names the
/// latter.
///
/// # Example
///
/// ```
/// use icepop_phf::Map;
///
/// let mut builder = Map::<&str, u32>::builder();
/// builder.insert("red", 0xff0000);
/// builder.insert("green", 0x00ff00);
///
/// let map = builder.build();
///
/// assert_eq!(map.get("green"), Some(&0x00ff00));
/// assert_eq!(map.get("blue"), None);
/// assert_eq!(map.len(), 2);
/// ```
pub struct Map<K, V, S = DefaultHasherSeed, P = NonPortable> {
    table: Table<MapOps<K, V>, S, P>,
}

/// Accumulates the entries of a [`Map`], then freezes them into one.
///
/// An ordinary mutable hash map until [`build`](Self::build) is called. Building is where the
/// minimal perfect hash function is constructed, so it is much more expensive than an insert
/// and should happen once, after all keys are known.
///
/// # Example
///
/// ```
/// use icepop_phf::map::Builder;
///
/// let mut builder = Builder::<&str, u32>::new();
/// builder.insert("a", 1);
/// assert_eq!(builder.insert("a", 2), Some(1));
///
/// let map = builder.build();
/// assert_eq!(map.get("a"), Some(&2));
/// ```
pub struct Builder<K, V, S = DefaultHasherSeed, P = NonPortable> {
    builder: TableBuilder<MapOps<K, V>, S, P>,
}

/// A [`Map`] that hashes and compares identically on every platform.
///
/// The form that supports `serde` and `rkyv`. See [`Portable`].
///
/// # Example
///
/// ```
/// use icepop_phf::PortableMap;
///
/// let map: PortableMap<&str, u32> = [("a", 1u32)].into_iter().collect();
///
/// assert_eq!(map.get("a"), Some(&1));
/// ```
pub type PortableMap<K, V, S = DefaultHasherSeed> = Map<K, V, S, Portable>;

/// The [`Builder`] that produces a [`PortableMap`].
///
/// # Example
///
/// ```
/// use icepop_phf::map::PortableBuilder;
///
/// let mut builder = PortableBuilder::<&str, u32>::new();
/// builder.insert("a", 1);
///
/// assert_eq!(builder.build().get("a"), Some(&1));
/// ```
pub type PortableBuilder<K, V, S = DefaultHasherSeed> = Builder<K, V, S, Portable>;

impl<K, V, S, P> Map<K, V, S, P> {
    /// Returns the hasher the map was built with.
    ///
    /// Lookups reuse it, so it is kept for the life of the map and travels with it through
    /// serialization.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{DefaultHasherSeed, Map};
    ///
    /// let map = Map::<&str, u32>::builder_with_hasher(DefaultHasherSeed::with_seed(42)).build();
    ///
    /// assert_eq!(map.hasher().seed(), 42);
    /// ```
    pub fn hasher(&self) -> &S {
        self.table.hasher()
    }

    /// Returns `true` if the map contains no entries.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Map;
    ///
    /// assert!(Map::<&str, u32>::builder().build().is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// Returns the number of entries in the map.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Map;
    ///
    /// let map: Map<&str, u32> = [("a", 1u32), ("b", 2)].into_iter().collect();
    ///
    /// assert_eq!(map.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Returns an iterator over the entries, in an arbitrary order.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Map;
    ///
    /// let map: Map<&str, u32> = [("a", 1u32), ("b", 2)].into_iter().collect();
    ///
    /// let mut entries: Vec<_> = map.iter().map(|(k, v)| (*k, *v)).collect();
    /// entries.sort();
    /// assert_eq!(entries, [("a", 1), ("b", 2)]);
    /// ```
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            iter: self.table.iter(),
        }
    }

    /// Returns an iterator over the entries with mutable values, in an arbitrary order.
    ///
    /// Keys are yielded by shared reference: changing one would invalidate the hash function.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Map;
    ///
    /// let mut map: Map<&str, u32> = [("a", 1u32), ("b", 2)].into_iter().collect();
    ///
    /// for (_, value) in map.iter_mut() {
    ///     *value *= 10;
    /// }
    ///
    /// assert_eq!(map.get("a"), Some(&10));
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        IterMut {
            iter: self.table.iter_mut(),
        }
    }

    /// Returns an iterator over the keys, in an arbitrary order.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Map;
    ///
    /// let map: Map<&str, u32> = [("a", 1u32), ("b", 2)].into_iter().collect();
    ///
    /// let mut keys: Vec<_> = map.keys().copied().collect();
    /// keys.sort();
    /// assert_eq!(keys, ["a", "b"]);
    /// ```
    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys {
            iter: self.table.iter(),
        }
    }

    /// Consumes the map and returns an iterator over its keys, in an arbitrary order.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Map;
    ///
    /// let map: Map<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    ///
    /// assert_eq!(map.into_keys().collect::<Vec<_>>(), ["a".to_string()]);
    /// ```
    pub fn into_keys(self) -> IntoKeys<K, V> {
        IntoKeys {
            iter: self.table.into_iter(),
        }
    }

    /// Returns an iterator over the values, in an arbitrary order.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Map;
    ///
    /// let map: Map<&str, u32> = [("a", 1u32), ("b", 2)].into_iter().collect();
    ///
    /// let mut values: Vec<_> = map.values().copied().collect();
    /// values.sort();
    /// assert_eq!(values, [1, 2]);
    /// ```
    pub fn values(&self) -> Values<'_, K, V> {
        Values {
            iter: self.table.iter(),
        }
    }

    /// Returns an iterator over the values with mutable access, in an arbitrary order.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Map;
    ///
    /// let mut map: Map<&str, u32> = [("a", 1u32)].into_iter().collect();
    ///
    /// for value in map.values_mut() {
    ///     *value += 1;
    /// }
    ///
    /// assert_eq!(map.get("a"), Some(&2));
    /// ```
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        ValuesMut {
            iter: self.table.iter_mut(),
        }
    }

    /// Consumes the map and returns an iterator over its values, in an arbitrary order.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Map;
    ///
    /// let map: Map<&str, u32> = [("a", 1u32)].into_iter().collect();
    ///
    /// assert_eq!(map.into_values().collect::<Vec<_>>(), [1]);
    /// ```
    pub fn into_values(self) -> IntoValues<K, V> {
        IntoValues {
            iter: self.table.into_iter(),
        }
    }

    /// Borrows the entries as a contiguous slice of pairs, in an arbitrary order.
    ///
    /// The order is fixed once the map is built, so an index obtained from
    /// [`get_index`](Self::get_index) addresses this slice.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Map;
    ///
    /// let map: Map<&str, u32> = [("a", 1u32), ("b", 2)].into_iter().collect();
    ///
    /// let mut entries = map.as_slice().to_vec();
    /// entries.sort();
    /// assert_eq!(entries, [("a", 1), ("b", 2)]);
    /// ```
    pub fn as_slice(&self) -> &[(K, V)] {
        self.table.as_slice()
    }

    /// Returns the entry at `index`, or `None` if it is out of bounds.
    ///
    /// Indices run over an arbitrary order, so this is only meaningful with an index from
    /// [`get_index`](Self::get_index) or from enumerating [`as_slice`](Self::as_slice).
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
    /// assert_eq!(map.index(2), None);
    /// ```
    pub fn index(&self, index: usize) -> Option<(&K, &V)> {
        self.table.map_index(index)
    }

    /// Returns the entry at `index` with a mutable value, or `None` if it is out of bounds.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Map;
    ///
    /// let mut map: Map<&str, u32> = [("a", 1u32)].into_iter().collect();
    /// let index = map.get_index("a").unwrap();
    ///
    /// *map.index_mut(index).unwrap().1 = 9;
    ///
    /// assert_eq!(map.get("a"), Some(&9));
    /// ```
    pub fn index_mut(&mut self, index: usize) -> Option<(&K, &mut V)> {
        self.table.map_index_mut(index)
    }

    /// Returns the entry at `index` without a bounds check.
    ///
    /// # Safety
    ///
    /// `index` must be less than [`len`](Self::len).
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Map;
    ///
    /// let map: Map<&str, u32> = [("a", 1u32)].into_iter().collect();
    /// let index = map.get_index("a").unwrap();
    ///
    /// // SAFETY: `get_index` returned this index, so it is in bounds.
    /// assert_eq!(unsafe { map.index_unchecked(index) }, (&"a", &1));
    /// ```
    pub unsafe fn index_unchecked(&self, index: usize) -> (&K, &V) {
        unsafe { self.table.map_index_unchecked(index) }
    }

    /// Returns the entry at `index` with a mutable value, without a bounds check.
    ///
    /// # Safety
    ///
    /// `index` must be less than [`len`](Self::len).
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Map;
    ///
    /// let mut map: Map<&str, u32> = [("a", 1u32)].into_iter().collect();
    /// let index = map.get_index("a").unwrap();
    ///
    /// // SAFETY: `get_index` returned this index, so it is in bounds.
    /// *unsafe { map.index_unchecked_mut(index) }.1 = 9;
    ///
    /// assert_eq!(map.get("a"), Some(&9));
    /// ```
    pub unsafe fn index_unchecked_mut(&mut self, index: usize) -> (&K, &mut V) {
        unsafe { self.table.map_index_unchecked_mut(index) }
    }
}

impl<K, V, S, P> core::fmt::Debug for Map<K, V, S, P>
where
    K: core::fmt::Debug,
    V: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<K, V, S, P> Clone for Map<K, V, S, P>
where
    K: Clone,
    V: Clone,
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            table: self.table.clone(),
        }
    }

    fn clone_from(&mut self, other: &Self) {
        self.table.clone_from(&other.table)
    }
}

impl<'a, K, V, S, P> IntoIterator for &'a Map<K, V, S, P> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V, S, P> IntoIterator for &'a mut Map<K, V, S, P> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<K, V, S, P> IntoIterator for Map<K, V, S, P> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            iter: self.table.into_iter(),
        }
    }
}

impl<K, V, S, P> Builder<K, V, S, P> {
    /// Returns the hasher the builder will hand to the map it builds.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{DefaultHasherSeed, Map};
    ///
    /// let builder = Map::<&str, u32>::builder_with_hasher(DefaultHasherSeed::with_seed(42));
    ///
    /// assert_eq!(builder.hasher().seed(), 42);
    /// ```
    pub fn hasher(&self) -> &S {
        self.builder.hasher()
    }

    /// Returns `true` if no entries have been inserted.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Map;
    ///
    /// let mut builder = Map::<&str, u32>::builder();
    /// assert!(builder.is_empty());
    ///
    /// builder.insert("a", 1);
    /// assert!(!builder.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.builder.is_empty()
    }

    /// Returns the number of entries inserted so far.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Map;
    ///
    /// let mut builder = Map::<&str, u32>::builder();
    /// builder.insert("a", 1);
    /// builder.insert("b", 2);
    ///
    /// assert_eq!(builder.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.builder.len()
    }

    /// Returns how many entries the builder can hold before it reallocates.
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
    pub fn capacity(&self) -> usize {
        self.builder.capacity()
    }

    /// Returns an iterator over the entries inserted so far, in an arbitrary order.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Map;
    ///
    /// let mut builder = Map::<&str, u32>::builder();
    /// builder.insert("a", 1);
    ///
    /// assert_eq!(builder.iter().collect::<Vec<_>>(), [(&"a", &1)]);
    /// ```
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            iter: self.builder.iter(),
        }
    }

    /// Returns an iterator over the inserted entries with mutable values.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Map;
    ///
    /// let mut builder = Map::<&str, u32>::builder();
    /// builder.insert("a", 1);
    ///
    /// for (_, value) in builder.iter_mut() {
    ///     *value = 9;
    /// }
    ///
    /// assert_eq!(builder.get("a"), Some(&9));
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        IterMut {
            iter: self.builder.iter_mut(),
        }
    }

    /// Returns an iterator over the inserted keys, in an arbitrary order.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Map;
    ///
    /// let mut builder = Map::<&str, u32>::builder();
    /// builder.insert("a", 1);
    ///
    /// assert_eq!(builder.keys().collect::<Vec<_>>(), [&"a"]);
    /// ```
    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys {
            iter: self.builder.iter(),
        }
    }

    /// Consumes the builder and returns an iterator over its keys, in an arbitrary order.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Map;
    ///
    /// let mut builder = Map::<&str, u32>::builder();
    /// builder.insert("a", 1);
    ///
    /// assert_eq!(builder.into_keys().collect::<Vec<_>>(), ["a"]);
    /// ```
    pub fn into_keys(self) -> IntoKeys<K, V> {
        IntoKeys {
            iter: self.builder.into_iter(),
        }
    }

    /// Returns an iterator over the inserted values, in an arbitrary order.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Map;
    ///
    /// let mut builder = Map::<&str, u32>::builder();
    /// builder.insert("a", 1);
    ///
    /// assert_eq!(builder.values().collect::<Vec<_>>(), [&1]);
    /// ```
    pub fn values(&self) -> Values<'_, K, V> {
        Values {
            iter: self.builder.iter(),
        }
    }

    /// Returns an iterator over the inserted values with mutable access.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Map;
    ///
    /// let mut builder = Map::<&str, u32>::builder();
    /// builder.insert("a", 1);
    ///
    /// for value in builder.values_mut() {
    ///     *value = 9;
    /// }
    ///
    /// assert_eq!(builder.get("a"), Some(&9));
    /// ```
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        ValuesMut {
            iter: self.builder.iter_mut(),
        }
    }

    /// Consumes the builder and returns an iterator over its values, in an arbitrary order.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Map;
    ///
    /// let mut builder = Map::<&str, u32>::builder();
    /// builder.insert("a", 1);
    ///
    /// assert_eq!(builder.into_values().collect::<Vec<_>>(), [1]);
    /// ```
    pub fn into_values(self) -> IntoValues<K, V> {
        IntoValues {
            iter: self.builder.into_iter(),
        }
    }

    /// Reserves room for at least `additional` more entries.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Map;
    ///
    /// let mut builder = Map::<&str, u32>::builder();
    /// builder.reserve(100);
    ///
    /// assert!(builder.capacity() >= 100);
    /// ```
    pub fn reserve(&mut self, additional: usize) {
        self.builder.reserve(additional);
    }

    /// Drops capacity beyond what the inserted entries need.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Map;
    ///
    /// let mut builder = Map::<&str, u32>::builder_with_capacity(100);
    /// builder.insert("a", 1);
    /// builder.shrink_to_fit();
    ///
    /// assert!(builder.capacity() < 100);
    /// ```
    pub fn shrink_to_fit(&mut self) {
        self.builder.shrink_to_fit();
    }

    /// Drops capacity down towards `min_capacity`, never below the current length.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Map;
    ///
    /// let mut builder = Map::<&str, u32>::builder_with_capacity(100);
    /// builder.insert("a", 1);
    /// builder.shrink_to(10);
    ///
    /// assert!(builder.capacity() < 100);
    /// ```
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.builder.shrink_to(min_capacity);
    }

    /// Removes every entry, keeping the allocated capacity.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Map;
    ///
    /// let mut builder = Map::<&str, u32>::builder();
    /// builder.insert("a", 1);
    /// builder.clear();
    ///
    /// assert!(builder.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.builder.clear();
    }
}

impl<K, V, S, P> core::fmt::Debug for Builder<K, V, S, P>
where
    K: core::fmt::Debug,
    V: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<K, V, S, P> Clone for Builder<K, V, S, P>
where
    K: Clone,
    V: Clone,
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            builder: self.builder.clone(),
        }
    }

    fn clone_from(&mut self, other: &Self) {
        self.builder.clone_from(&other.builder);
    }
}

impl<'a, K, V, S, P> IntoIterator for &'a Builder<K, V, S, P> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V, S, P> IntoIterator for &'a mut Builder<K, V, S, P> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<K, V, S, P> IntoIterator for Builder<K, V, S, P> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            iter: self.builder.into_iter(),
        }
    }
}
