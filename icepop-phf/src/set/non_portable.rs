//! Lookup and build interface for a [`Set`] parameterized by [`NonPortable`].

use super::{Builder, Set};
use crate::portability::NonPortable;

use equivalent::Equivalent;
use portable::DefaultHasherSeed;

use core::hash::{BuildHasher, Hash};

impl<T> Set<T, DefaultHasherSeed, NonPortable> {
    /// Creates an empty [`Builder`] with a fresh [`DefaultHasherSeed`].
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Set;
    ///
    /// let mut builder = Set::<u32>::builder();
    /// builder.insert(1);
    ///
    /// assert_eq!(builder.build().len(), 1);
    /// ```
    pub fn builder() -> Builder<T, DefaultHasherSeed, NonPortable> {
        Builder::<T, DefaultHasherSeed, NonPortable>::new()
    }

    /// Creates an empty [`Builder`] with room for `capacity` elements.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Set;
    ///
    /// let builder = Set::<u32>::builder_with_capacity(10);
    ///
    /// assert!(builder.capacity() >= 10);
    /// ```
    pub fn builder_with_capacity(capacity: usize) -> Builder<T, DefaultHasherSeed, NonPortable> {
        Builder::<T, DefaultHasherSeed, NonPortable>::with_capacity(capacity)
    }
}

impl<T, S> Set<T, S, NonPortable>
where
    S: BuildHasher,
{
    /// Creates an empty [`Builder`] that will hash with `hasher`.
    ///
    /// The hasher is stored in the finished set and reused for every lookup, so it must produce
    /// the same hash for the same element for as long as the set is read.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{DefaultHasherSeed, Set};
    ///
    /// let builder = Set::<u32>::builder_with_hasher(DefaultHasherSeed::with_seed(7));
    ///
    /// assert_eq!(builder.hasher().seed(), 7);
    /// ```
    pub fn builder_with_hasher(hasher: S) -> Builder<T, S, NonPortable> {
        Builder::<T, S, NonPortable>::with_hasher(hasher)
    }

    /// Creates an empty [`Builder`] with room for `capacity` elements, hashing with `hasher`.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{DefaultHasherSeed, Set};
    ///
    /// let builder =
    ///     Set::<u32>::builder_with_capacity_and_hasher(10, DefaultHasherSeed::with_seed(7));
    ///
    /// assert!(builder.capacity() >= 10);
    /// assert_eq!(builder.hasher().seed(), 7);
    /// ```
    pub fn builder_with_capacity_and_hasher(
        capacity: usize,
        hasher: S,
    ) -> Builder<T, S, NonPortable> {
        Builder::<T, S, NonPortable>::with_capacity_and_hasher(capacity, hasher)
    }

    /// Returns the index `key` hashes to, without confirming that it is present.
    ///
    /// # Safety
    ///
    /// The set must not be empty.
    ///
    /// Looking up an element that is not present returns an arbitrary in-bounds index rather
    /// than failing.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Set;
    ///
    /// let set: Set<u32> = [1u32, 2, 3].into_iter().collect();
    ///
    /// // SAFETY: the set is not empty.
    /// let index = unsafe { set.get_index_unchecked(&2u32) };
    ///
    /// assert_eq!(set.index(index), Some(&2));
    /// ```
    pub unsafe fn get_index_unchecked<Q>(&self, key: &Q) -> usize
    where
        Q: Hash + Equivalent<T> + ?Sized,
    {
        // SAFETY: `Table::get_index_unchecked` has this function's contract, which the caller
        // upheld.
        unsafe { self.table.get_index_unchecked(key) }
    }

    /// Returns the index of the element equal to `key`, or `None` if there is none.
    ///
    /// The index addresses [`as_slice`](Self::as_slice) and [`index`](Self::index), and stays
    /// valid for the life of the set.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Set;
    ///
    /// let set: Set<u32> = [1u32, 2, 3].into_iter().collect();
    /// let index = set.get_index(&2u32).unwrap();
    ///
    /// assert_eq!(set.index(index), Some(&2));
    /// assert_eq!(set.get_index(&9u32), None);
    /// ```
    pub fn get_index<Q>(&self, key: &Q) -> Option<usize>
    where
        Q: Hash + Equivalent<T> + ?Sized,
    {
        self.table.get_index(key)
    }

    /// Returns `true` if the set contains an element equal to `key`.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Set;
    ///
    /// let set: Set<String> = ["ash".to_string(), "elm".to_string()].into_iter().collect();
    ///
    /// assert!(set.contains("ash"));
    /// assert!(!set.contains("oak"));
    /// ```
    pub fn contains<Q>(&self, key: &Q) -> bool
    where
        Q: Hash + Equivalent<T> + ?Sized,
    {
        self.table.contains(key)
    }

    /// Returns the stored element equal to `key`, or `None` if there is none.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Set;
    ///
    /// let set: Set<String> = ["ash".to_string()].into_iter().collect();
    ///
    /// assert_eq!(set.get("ash").map(String::as_str), Some("ash"));
    /// assert_eq!(set.get("oak"), None);
    /// ```
    pub fn get<Q>(&self, key: &Q) -> Option<&T>
    where
        Q: Hash + Equivalent<T> + ?Sized,
    {
        self.table.get(key)
    }

    /// Returns the element `key` hashes to, without confirming that it is equal.
    ///
    /// # Safety
    ///
    /// The set must not be empty.
    ///
    /// Looking up an element that is not present returns an arbitrary element rather than
    /// failing.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Set;
    ///
    /// let set: Set<u32> = [1u32, 2, 3].into_iter().collect();
    ///
    /// // SAFETY: the set is not empty.
    /// assert_eq!(unsafe { set.get_unchecked(&2u32) }, &2);
    /// ```
    pub unsafe fn get_unchecked<Q>(&self, key: &Q) -> &T
    where
        Q: Hash + Equivalent<T> + ?Sized,
    {
        // SAFETY: `Table::get_unchecked` has this function's contract, which the caller upheld.
        unsafe { self.table.get_unchecked(key) }
    }
}

impl<T, S> Default for Set<T, S, NonPortable>
where
    S: Default,
{
    fn default() -> Self {
        Self {
            table: super::Table::default(),
        }
    }
}

impl<T, S> FromIterator<T> for Set<T, S, NonPortable>
where
    S: Default + BuildHasher,
    T: Hash + Eq,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Builder::<T, S, NonPortable>::from_iter(iter).build()
    }
}

impl<T, S> PartialEq for Set<T, S, NonPortable>
where
    T: Hash + Eq,
    S: BuildHasher,
{
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter().all(|entry| other.contains(entry))
    }
}

impl<T, S> Eq for Set<T, S, NonPortable>
where
    T: Hash + Eq,
    S: BuildHasher,
{
}

impl<T> Builder<T, DefaultHasherSeed, NonPortable> {
    /// Creates an empty builder with a fresh [`DefaultHasherSeed`].
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::set::Builder;
    ///
    /// let mut builder = Builder::<u32>::new();
    /// builder.insert(1);
    ///
    /// assert_eq!(builder.len(), 1);
    /// ```
    pub fn new() -> Self {
        Self::with_hasher(DefaultHasherSeed::new())
    }

    /// Creates an empty builder with room for `capacity` elements.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::set::Builder;
    ///
    /// let builder = Builder::<u32>::with_capacity(10);
    ///
    /// assert!(builder.capacity() >= 10);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, DefaultHasherSeed::new())
    }
}

impl<T, S> Builder<T, S, NonPortable> {
    /// Creates an empty builder that will hash with `hasher`.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{DefaultHasherSeed, set::Builder};
    ///
    /// let builder = Builder::<u32>::with_hasher(DefaultHasherSeed::with_seed(7));
    ///
    /// assert_eq!(builder.hasher().seed(), 7);
    /// ```
    pub fn with_hasher(hasher: S) -> Self {
        Self {
            builder: super::TableBuilder::with_hasher(hasher),
        }
    }

    /// Creates an empty builder with room for `capacity` elements, hashing with `hasher`.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{DefaultHasherSeed, set::Builder};
    ///
    /// let builder =
    ///     Builder::<u32>::with_capacity_and_hasher(10, DefaultHasherSeed::with_seed(7));
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

impl<T, S> Builder<T, S, NonPortable>
where
    S: BuildHasher,
{
    /// Returns `true` if an element equal to `key` has been inserted.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::set::Builder;
    ///
    /// let mut builder = Builder::<u32>::new();
    /// builder.insert(1);
    ///
    /// assert!(builder.contains(&1u32));
    /// assert!(!builder.contains(&2u32));
    /// ```
    pub fn contains<Q>(&self, key: &Q) -> bool
    where
        Q: Hash + Equivalent<T> + ?Sized,
    {
        self.builder.contains(key)
    }

    /// Returns the inserted element equal to `key`, or `None` if there is none.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::set::Builder;
    ///
    /// let mut builder = Builder::<String>::new();
    /// builder.insert("ash".to_string());
    ///
    /// assert_eq!(builder.get("ash").map(String::as_str), Some("ash"));
    /// assert_eq!(builder.get("oak"), None);
    /// ```
    pub fn get<Q>(&self, key: &Q) -> Option<&T>
    where
        Q: Hash + Equivalent<T> + ?Sized,
    {
        self.builder.get(key)
    }

    /// Returns the element equal to `key`, inserting `default()` first if there is none.
    ///
    /// `default` is not called when the element is already present, which is what distinguishes
    /// this from [`get_or_insert`](Self::get_or_insert) when producing the element is expensive.
    ///
    /// # Panics
    ///
    /// Panics if `default()` returns a value that is not equal to `key`, or if the builder
    /// already holds `u32::MAX` elements.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::set::Builder;
    ///
    /// let mut builder = Builder::<String>::new();
    ///
    /// assert_eq!(builder.get_or_insert_with("ash", || "ash".to_string()), "ash");
    /// assert_eq!(builder.len(), 1);
    ///
    /// // Already present, so the closure is not called.
    /// assert_eq!(builder.get_or_insert_with("ash", || unreachable!()), "ash");
    /// assert_eq!(builder.len(), 1);
    /// ```
    pub fn get_or_insert_with<Q, F>(&mut self, key: &Q, default: F) -> &T
    where
        Q: Hash + Equivalent<T> + ?Sized,
        F: FnOnce() -> T,
    {
        self.builder.get_or_insert_with(key, default)
    }
}

impl<T, S> Builder<T, S, NonPortable>
where
    T: Hash + Eq,
    S: BuildHasher,
{
    /// Inserts `value` and returns `true`, or leaves an equal element in place and returns
    /// `false`.
    ///
    /// # Panics
    ///
    /// Panics if the builder already holds `u32::MAX` elements.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::set::Builder;
    ///
    /// let mut builder = Builder::<u32>::new();
    ///
    /// assert!(builder.insert(1));
    /// assert!(!builder.insert(1));
    /// assert_eq!(builder.len(), 1);
    /// ```
    pub fn insert(&mut self, value: T) -> bool {
        self.builder.insert(value)
    }

    /// Inserts `value`, returning the equal element it displaced.
    ///
    /// Unlike [`insert`](Self::insert), the new value wins. That matters when `T` carries data
    /// its equality ignores.
    ///
    /// # Panics
    ///
    /// Panics if the builder already holds `u32::MAX` elements.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::set::Builder;
    ///
    /// let mut builder = Builder::<u32>::new();
    ///
    /// assert_eq!(builder.replace(1), None);
    /// assert_eq!(builder.replace(1), Some(1));
    /// assert_eq!(builder.len(), 1);
    /// ```
    pub fn replace(&mut self, value: T) -> Option<T> {
        self.builder.replace(value)
    }

    /// Returns the element equal to `value`, inserting `value` first if there is none.
    ///
    /// # Panics
    ///
    /// Panics if the builder already holds `u32::MAX` elements.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::set::Builder;
    ///
    /// let mut builder = Builder::<u32>::new();
    ///
    /// assert_eq!(builder.get_or_insert(1), &1);
    /// assert_eq!(builder.get_or_insert(1), &1);
    /// assert_eq!(builder.len(), 1);
    /// ```
    pub fn get_or_insert(&mut self, value: T) -> &T {
        self.builder.get_or_insert(value)
    }

    /// Removes and returns the element equal to `key`, or `None` if there is none.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::set::Builder;
    ///
    /// let mut builder = Builder::<u32>::new();
    /// builder.insert(1);
    ///
    /// assert_eq!(builder.take(&1u32), Some(1));
    /// assert_eq!(builder.take(&1u32), None);
    /// ```
    pub fn take<Q>(&mut self, key: &Q) -> Option<T>
    where
        Q: Hash + Equivalent<T> + ?Sized,
    {
        self.builder.take(key)
    }

    /// Removes the element equal to `key`, returning whether one was there.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::set::Builder;
    ///
    /// let mut builder = Builder::<u32>::new();
    /// builder.insert(1);
    ///
    /// assert!(builder.remove(&1u32));
    /// assert!(!builder.remove(&1u32));
    /// ```
    pub fn remove<Q>(&mut self, key: &Q) -> bool
    where
        Q: Hash + Equivalent<T> + ?Sized,
    {
        self.builder.remove(key)
    }

    /// Constructs the minimal perfect hash function and freezes the builder into a [`Set`].
    ///
    /// This is where the cost of the collection is paid; everything before it is an ordinary
    /// hash set.
    ///
    /// # Panics
    ///
    /// Panics if no minimal perfect hash function can be constructed for the inserted elements.
    /// That means the hasher gave two distinct elements the same 64-bit hash under every
    /// parameter it was retried with, or distributed them too poorly to place.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::set::Builder;
    ///
    /// let mut builder = Builder::<u32>::new();
    /// builder.insert(1);
    /// builder.insert(2);
    ///
    /// let set = builder.build();
    ///
    /// assert!(set.contains(&1u32));
    /// ```
    pub fn build(self) -> Set<T, S, NonPortable> {
        Set {
            table: self.builder.build(),
        }
    }
}

impl<T, S> Default for Builder<T, S, NonPortable>
where
    S: Default,
{
    fn default() -> Self {
        Self::with_hasher(S::default())
    }
}

impl<T, S> Extend<T> for Builder<T, S, NonPortable>
where
    S: BuildHasher,
    T: Hash + Eq,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        let iter = iter.into_iter();
        let cap = iter.size_hint().0;
        self.reserve(cap);
        iter.into_iter().for_each(|item| {
            self.insert(item);
        });
    }
}

impl<T, S> FromIterator<T> for Builder<T, S, NonPortable>
where
    S: Default + BuildHasher,
    T: Hash + Eq,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let iter = iter.into_iter();
        let cap = iter.size_hint().0;
        let mut builder = Self::with_capacity_and_hasher(cap, S::default());
        iter.for_each(|item| {
            builder.insert(item);
        });
        builder
    }
}

impl<T, S> PartialEq for Builder<T, S, NonPortable>
where
    T: Hash + Eq,
    S: BuildHasher,
{
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter().all(|entry| other.contains(entry))
    }
}

impl<T, S> Eq for Builder<T, S, NonPortable>
where
    T: Hash + Eq,
    S: BuildHasher,
{
}
