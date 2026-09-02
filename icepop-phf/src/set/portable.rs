//! Lookup and build interface for a [`Set`] parameterized by [`Portable`].

use super::{Builder, Set};
use crate::portability::Portable;

use portable::{DefaultHasherSeed, PortableBuildHasher, PortableEq, PortableHash};

impl<T> Set<T, DefaultHasherSeed, Portable> {
    /// Creates an empty [`Builder`] with a fresh [`DefaultHasherSeed`].
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::PortableSet;
    ///
    /// let mut builder = PortableSet::<u32>::builder();
    /// builder.insert(1);
    ///
    /// assert_eq!(builder.build().len(), 1);
    /// ```
    pub fn builder() -> Builder<T, DefaultHasherSeed, Portable> {
        Builder::<T, DefaultHasherSeed, Portable>::new()
    }

    /// Creates an empty [`Builder`] with room for `capacity` elements.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::PortableSet;
    ///
    /// let builder = PortableSet::<u32>::builder_with_capacity(10);
    ///
    /// assert!(builder.capacity() >= 10);
    /// ```
    pub fn builder_with_capacity(capacity: usize) -> Builder<T, DefaultHasherSeed, Portable> {
        Builder::<T, DefaultHasherSeed, Portable>::with_capacity(capacity)
    }
}

impl<T, S> Set<T, S, Portable>
where
    S: PortableBuildHasher,
{
    /// Creates an empty [`Builder`] that will hash with `hasher`.
    ///
    /// The hasher is stored in the finished set and reused for every lookup, so it must produce
    /// the same hash for the same element for as long as the set is read.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{DefaultHasherSeed, PortableSet};
    ///
    /// let builder = PortableSet::<u32>::builder_with_hasher(DefaultHasherSeed::with_seed(7));
    ///
    /// assert_eq!(builder.hasher().seed(), 7);
    /// ```
    pub fn builder_with_hasher(hasher: S) -> Builder<T, S, Portable> {
        Builder::<T, S, Portable>::with_hasher(hasher)
    }

    /// Creates an empty [`Builder`] with room for `capacity` elements, hashing with `hasher`.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{DefaultHasherSeed, PortableSet};
    ///
    /// let builder =
    ///     PortableSet::<u32>::builder_with_capacity_and_hasher(10, DefaultHasherSeed::with_seed(7));
    ///
    /// assert!(builder.capacity() >= 10);
    /// assert_eq!(builder.hasher().seed(), 7);
    /// ```
    pub fn builder_with_capacity_and_hasher(capacity: usize, hasher: S) -> Builder<T, S, Portable> {
        Builder::<T, S, Portable>::with_capacity_and_hasher(capacity, hasher)
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
    /// use icepop_phf::PortableSet;
    ///
    /// let set: PortableSet<u32> = [1u32, 2, 3].into_iter().collect();
    ///
    /// // SAFETY: the set is not empty.
    /// let index = unsafe { set.get_index_unchecked(&2u32) };
    ///
    /// assert_eq!(set.index(index), Some(&2));
    /// ```
    pub unsafe fn get_index_unchecked<Q>(&self, key: &Q) -> usize
    where
        Q: PortableHash + PortableEq<T> + ?Sized,
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
    /// use icepop_phf::PortableSet;
    ///
    /// let set: PortableSet<u32> = [1u32, 2, 3].into_iter().collect();
    /// let index = set.get_index(&2u32).unwrap();
    ///
    /// assert_eq!(set.index(index), Some(&2));
    /// assert_eq!(set.get_index(&9u32), None);
    /// ```
    pub fn get_index<Q>(&self, key: &Q) -> Option<usize>
    where
        Q: PortableHash + PortableEq<T> + ?Sized,
    {
        self.table.get_index(key)
    }

    /// Returns `true` if the set contains an element equal to `key`.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::PortableSet;
    ///
    /// let set: PortableSet<String> = ["ash".to_string(), "elm".to_string()].into_iter().collect();
    ///
    /// assert!(set.contains("ash"));
    /// assert!(!set.contains("oak"));
    /// ```
    pub fn contains<Q>(&self, key: &Q) -> bool
    where
        Q: PortableHash + PortableEq<T> + ?Sized,
    {
        self.table.contains(key)
    }

    /// Returns the stored element equal to `key`, or `None` if there is none.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::PortableSet;
    ///
    /// let set: PortableSet<String> = ["ash".to_string()].into_iter().collect();
    ///
    /// assert_eq!(set.get("ash").map(String::as_str), Some("ash"));
    /// assert_eq!(set.get("oak"), None);
    /// ```
    pub fn get<Q>(&self, key: &Q) -> Option<&T>
    where
        Q: PortableHash + PortableEq<T> + ?Sized,
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
    /// use icepop_phf::PortableSet;
    ///
    /// let set: PortableSet<u32> = [1u32, 2, 3].into_iter().collect();
    ///
    /// // SAFETY: the set is not empty.
    /// assert_eq!(unsafe { set.get_unchecked(&2u32) }, &2);
    /// ```
    pub unsafe fn get_unchecked<Q>(&self, key: &Q) -> &T
    where
        Q: PortableHash + PortableEq<T> + ?Sized,
    {
        // SAFETY: `Table::get_unchecked` has this function's contract, which the caller upheld.
        unsafe { self.table.get_unchecked(key) }
    }
}

impl<T, S> Default for Set<T, S, Portable>
where
    S: Default,
{
    fn default() -> Self {
        Self {
            table: super::Table::default(),
        }
    }
}

impl<T, S> FromIterator<T> for Set<T, S, Portable>
where
    S: Default + PortableBuildHasher,
    T: PortableHash + PortableEq,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Builder::<T, S, Portable>::from_iter(iter).build()
    }
}

impl<T, S> PartialEq for Set<T, S, Portable>
where
    T: PortableHash + PortableEq + PartialEq,
    S: PortableBuildHasher,
{
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter()
            .all(|entry| other.get(entry).is_some_and(|other| entry == other))
    }
}

impl<T, S> Eq for Set<T, S, Portable>
where
    T: PortableHash + PortableEq + Eq,
    S: PortableBuildHasher,
{
}

impl<T> Builder<T, DefaultHasherSeed, Portable> {
    /// Creates an empty builder with a fresh [`DefaultHasherSeed`].
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::set::PortableBuilder;
    ///
    /// let mut builder = PortableBuilder::<u32>::new();
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
    /// use icepop_phf::set::PortableBuilder;
    ///
    /// let builder = PortableBuilder::<u32>::with_capacity(10);
    ///
    /// assert!(builder.capacity() >= 10);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, DefaultHasherSeed::new())
    }
}

impl<T, S> Builder<T, S, Portable> {
    /// Creates an empty builder that will hash with `hasher`.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{DefaultHasherSeed, set::PortableBuilder};
    ///
    /// let builder = PortableBuilder::<u32>::with_hasher(DefaultHasherSeed::with_seed(7));
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
    /// use icepop_phf::{DefaultHasherSeed, set::PortableBuilder};
    ///
    /// let builder =
    ///     PortableBuilder::<u32>::with_capacity_and_hasher(10, DefaultHasherSeed::with_seed(7));
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

impl<T, S> Builder<T, S, Portable>
where
    S: PortableBuildHasher,
{
    /// Returns `true` if an element equal to `key` has been inserted.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::set::PortableBuilder;
    ///
    /// let mut builder = PortableBuilder::<u32>::new();
    /// builder.insert(1);
    ///
    /// assert!(builder.contains(&1u32));
    /// assert!(!builder.contains(&2u32));
    /// ```
    pub fn contains<Q>(&self, key: &Q) -> bool
    where
        Q: PortableHash + PortableEq<T> + ?Sized,
    {
        self.builder.contains(key)
    }

    /// Returns the inserted element equal to `key`, or `None` if there is none.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::set::PortableBuilder;
    ///
    /// let mut builder = PortableBuilder::<String>::new();
    /// builder.insert("ash".to_string());
    ///
    /// assert_eq!(builder.get("ash").map(String::as_str), Some("ash"));
    /// assert_eq!(builder.get("oak"), None);
    /// ```
    pub fn get<Q>(&self, key: &Q) -> Option<&T>
    where
        Q: PortableHash + PortableEq<T> + ?Sized,
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
    /// use icepop_phf::set::PortableBuilder;
    ///
    /// let mut builder = PortableBuilder::<String>::new();
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
        Q: PortableHash + PortableEq<T> + ?Sized,
        F: FnOnce() -> T,
    {
        self.builder.get_or_insert_with(key, default)
    }
}

impl<T, S> Builder<T, S, Portable>
where
    T: PortableHash + PortableEq,
    S: PortableBuildHasher,
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
    /// use icepop_phf::set::PortableBuilder;
    ///
    /// let mut builder = PortableBuilder::<u32>::new();
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
    /// use icepop_phf::set::PortableBuilder;
    ///
    /// let mut builder = PortableBuilder::<u32>::new();
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
    /// use icepop_phf::set::PortableBuilder;
    ///
    /// let mut builder = PortableBuilder::<u32>::new();
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
    /// use icepop_phf::set::PortableBuilder;
    ///
    /// let mut builder = PortableBuilder::<u32>::new();
    /// builder.insert(1);
    ///
    /// assert_eq!(builder.take(&1u32), Some(1));
    /// assert_eq!(builder.take(&1u32), None);
    /// ```
    pub fn take<Q>(&mut self, key: &Q) -> Option<T>
    where
        Q: PortableHash + PortableEq<T> + ?Sized,
    {
        self.builder.take(key)
    }

    /// Removes the element equal to `key`, returning whether one was there.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::set::PortableBuilder;
    ///
    /// let mut builder = PortableBuilder::<u32>::new();
    /// builder.insert(1);
    ///
    /// assert!(builder.remove(&1u32));
    /// assert!(!builder.remove(&1u32));
    /// ```
    pub fn remove<Q>(&mut self, key: &Q) -> bool
    where
        Q: PortableHash + PortableEq<T> + ?Sized,
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
    /// use icepop_phf::set::PortableBuilder;
    ///
    /// let mut builder = PortableBuilder::<u32>::new();
    /// builder.insert(1);
    /// builder.insert(2);
    ///
    /// let set = builder.build();
    ///
    /// assert!(set.contains(&1u32));
    /// ```
    pub fn build(self) -> Set<T, S, Portable> {
        Set {
            table: self.builder.build(),
        }
    }
}

impl<T, S> Default for Builder<T, S, Portable>
where
    S: Default,
{
    fn default() -> Self {
        Self::with_hasher(S::default())
    }
}

impl<T, S> Extend<T> for Builder<T, S, Portable>
where
    S: PortableBuildHasher,
    T: PortableHash + PortableEq,
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

impl<T, S> FromIterator<T> for Builder<T, S, Portable>
where
    S: Default + PortableBuildHasher,
    T: PortableHash + PortableEq,
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

impl<T, S> PartialEq for Builder<T, S, Portable>
where
    T: PortableHash + PortableEq + PartialEq,
    S: PortableBuildHasher,
{
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter()
            .all(|entry| other.get(entry).is_some_and(|other| entry == other))
    }
}

impl<T, S> Eq for Builder<T, S, Portable>
where
    T: PortableHash + PortableEq + Eq,
    S: PortableBuildHasher,
{
}

cfg_select!(feature = "serde" => {
    impl<T, S> serde::Serialize for Set<T, S, Portable>
    where
        T: serde::Serialize,
        S: serde::Serialize,
    {
        fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
        where
            Ser: serde::Serializer,
        {
            self.table.serialize(serializer)
        }
    }

    impl<'de, T, S> serde::Deserialize<'de> for Set<T, S, Portable>
    where
        T: serde::Deserialize<'de>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PortableSet;

    use alloc::vec::Vec;

    fn set_of(keys: impl IntoIterator<Item = u32>) -> PortableSet<u32> {
        let mut builder = PortableSet::<u32>::builder_with_hasher(DefaultHasherSeed::with_seed(1));
        for k in keys {
            builder.insert(k);
        }
        builder.build()
    }

    #[test]
    fn every_constructor_reaches_the_same_builder() {
        assert!(PortableSet::<u32>::builder().build().is_empty());
        assert!(PortableSet::<u32>::builder_with_capacity(8).capacity() >= 8);
        assert_eq!(
            PortableSet::<u32>::builder_with_hasher(DefaultHasherSeed::with_seed(3))
                .hasher()
                .seed(),
            3,
        );

        let builder = PortableSet::<u32>::builder_with_capacity_and_hasher(
            8,
            DefaultHasherSeed::with_seed(3),
        );
        assert!(builder.capacity() >= 8);
        assert_eq!(builder.hasher().seed(), 3);

        assert!(
            Builder::<u32, DefaultHasherSeed, Portable>::new()
                .build()
                .is_empty()
        );
        assert!(Builder::<u32, DefaultHasherSeed, Portable>::with_capacity(8).capacity() >= 8);
        assert_eq!(
            Builder::<u32, DefaultHasherSeed, Portable>::with_hasher(DefaultHasherSeed::with_seed(
                3
            ))
            .hasher()
            .seed(),
            3,
        );
        assert!(
            Builder::<u32, DefaultHasherSeed, Portable>::with_capacity_and_hasher(
                8,
                DefaultHasherSeed::with_seed(3)
            )
            .capacity()
                >= 8
        );

        assert!(PortableSet::<u32>::default().is_empty());
        assert!(
            Builder::<u32, DefaultHasherSeed, Portable>::default()
                .build()
                .is_empty()
        );
    }

    #[test]
    fn lookups_reach_present_and_absent_keys() {
        let set = set_of(0..8);

        for k in 0..8u32 {
            let index = set.get_index(&k).unwrap();
            assert!(set.contains(&k));
            assert_eq!(set.get(&k), Some(&k));
            assert_eq!(set.index(index), Some(&k));

            // SAFETY: the set is not empty.
            unsafe {
                assert_eq!(set.get_index_unchecked(&k), index);
                assert_eq!(set.get_unchecked(&k), &k);
            }
        }

        assert_eq!(set.get_index(&99u32), None);
        assert!(!set.contains(&99u32));
        assert_eq!(set.get(&99u32), None);
    }

    #[test]
    fn the_builder_inserts_replaces_and_removes() {
        let mut builder = PortableSet::<u32>::builder();

        assert!(builder.insert(1));
        assert!(!builder.insert(1));
        assert_eq!(builder.replace(1), Some(1));
        assert_eq!(builder.get_or_insert(2), &2);
        assert_eq!(builder.get_or_insert_with(&3u32, || 3), &3);
        assert_eq!(builder.get_or_insert_with(&3u32, || unreachable!()), &3);

        assert!(builder.contains(&1u32));
        assert_eq!(builder.get(&2u32), Some(&2));
        assert_eq!(builder.get(&99u32), None);
        assert!(!builder.contains(&99u32));

        assert_eq!(builder.take(&1u32), Some(1));
        assert_eq!(builder.take(&1u32), None);
        assert!(builder.remove(&2u32));
        assert!(!builder.remove(&2u32));
        assert_eq!(builder.len(), 1);
    }

    #[test]
    fn collecting_keeps_the_first_of_two_equal_elements() {
        // `insert` semantics, matching the standard library's `HashSet`.
        let set: PortableSet<u32> = [1u32, 2, 1].into_iter().collect();
        assert_eq!(set.len(), 2);

        let mut builder: Builder<u32, DefaultHasherSeed, Portable> =
            [1u32, 2].into_iter().collect();
        builder.extend([2u32, 3]);
        assert_eq!(builder.len(), 3);

        let mut all = builder.build().iter().copied().collect::<Vec<_>>();
        all.sort_unstable();
        assert_eq!(all, [1, 2, 3]);
    }

    #[test]
    fn equality_ignores_the_element_order_and_the_hasher() {
        let a = set_of(0..8);
        let mut b = PortableSet::<u32>::builder_with_hasher(DefaultHasherSeed::with_seed(9));
        for k in (0..8u32).rev() {
            b.insert(k);
        }
        let b = b.build();

        assert_eq!(a, b);
        assert_eq!(a, a);
        assert_ne!(a, set_of(0..7));
        assert_ne!(a, set_of(1..9));

        let mut ba = PortableSet::<u32>::builder();
        let mut bb = PortableSet::<u32>::builder();
        ba.insert(1);
        bb.insert(1);
        assert_eq!(ba, bb);
        bb.insert(2);
        assert_ne!(ba, bb);
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use super::*;
    use crate::PortableSet;

    use serde_test::{Token, assert_de_tokens, assert_de_tokens_error, assert_ser_tokens};

    /// A one-entry set, whose layout is fixed whatever the hasher does: the only slot is the
    /// only index, so `entries` and `params` cannot come out any other way.
    fn one_entry() -> PortableSet<u32> {
        let mut builder = PortableSet::<u32>::builder_with_hasher(DefaultHasherSeed::with_seed(7));
        builder.insert(42);
        builder.build()
    }

    fn hasher_tokens() -> [Token; 4] {
        [
            Token::Struct {
                name: "DefaultHasherSeed",
                len: 1,
            },
            Token::Str("seed"),
            Token::U64(7),
            Token::StructEnd,
        ]
    }

    #[test]
    fn a_set_without_a_slot_table_writes_four_fields() {
        assert_ser_tokens(
            &one_entry(),
            &[
                Token::Struct {
                    name: "Table",
                    len: 4,
                },
                Token::Str("hasher"),
                hasher_tokens()[0],
                hasher_tokens()[1],
                hasher_tokens()[2],
                hasher_tokens()[3],
                Token::Str("entries"),
                Token::Seq { len: Some(1) },
                Token::U32(42),
                Token::SeqEnd,
                Token::Str("global_param"),
                Token::U8(0),
                Token::Str("params"),
                Token::Seq { len: Some(1) },
                Token::U32(0),
                Token::SeqEnd,
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn a_compact_format_may_send_the_fields_as_a_sequence() {
        // A non-self-describing format drops the names entirely, which is the `visit_seq` path.
        assert_de_tokens(
            &one_entry(),
            &[
                Token::Seq { len: Some(4) },
                hasher_tokens()[0],
                hasher_tokens()[1],
                hasher_tokens()[2],
                hasher_tokens()[3],
                Token::Seq { len: Some(1) },
                Token::U32(42),
                Token::SeqEnd,
                Token::U8(0),
                Token::Seq { len: Some(1) },
                Token::U32(0),
                Token::SeqEnd,
                Token::SeqEnd,
            ],
        );
    }

    #[test]
    fn a_format_may_identify_the_fields_by_index() {
        assert_de_tokens(
            &one_entry(),
            &[
                Token::Map { len: Some(4) },
                Token::U64(0),
                hasher_tokens()[0],
                hasher_tokens()[1],
                hasher_tokens()[2],
                hasher_tokens()[3],
                Token::U64(1),
                Token::Seq { len: Some(1) },
                Token::U32(42),
                Token::SeqEnd,
                Token::U64(2),
                Token::U8(0),
                Token::U64(3),
                Token::Seq { len: Some(1) },
                Token::U32(0),
                Token::SeqEnd,
                Token::MapEnd,
            ],
        );
    }

    #[test]
    fn a_field_index_outside_the_struct_is_refused() {
        assert_de_tokens_error::<PortableSet<u32>>(
            &[Token::Map { len: Some(1) }, Token::U64(9)],
            "invalid value: integer `9`, expected one of \"global_param\", \"params\", \
             \"indices\", \"entries\", or \"hasher\"",
        );
    }

    #[test]
    fn a_sequence_that_stops_early_is_refused() {
        assert_de_tokens_error::<PortableSet<u32>>(
            &[
                Token::Seq { len: Some(1) },
                hasher_tokens()[0],
                hasher_tokens()[1],
                hasher_tokens()[2],
                hasher_tokens()[3],
                Token::SeqEnd,
            ],
            "missing field `entries`",
        );
    }
}
