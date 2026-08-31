use super::{Builder, Set};
use crate::portability::Portable;

use portable::{DefaultHasherSeed, PortableBuildHasher, PortableEq, PortableHash};

impl<T> Set<T, DefaultHasherSeed, Portable> {
    pub fn builder() -> Builder<T, DefaultHasherSeed, Portable> {
        Builder::<T, DefaultHasherSeed, Portable>::new()
    }

    pub fn builder_with_capacity(capacity: usize) -> Builder<T, DefaultHasherSeed, Portable> {
        Builder::<T, DefaultHasherSeed, Portable>::with_capacity(capacity)
    }
}

impl<T, S> Set<T, S, Portable>
where
    S: PortableBuildHasher,
{
    pub fn builder_with_hasher(hasher: S) -> Builder<T, S, Portable> {
        Builder::<T, S, Portable>::with_hasher(hasher)
    }

    pub fn builder_with_capacity_and_hasher(capacity: usize, hasher: S) -> Builder<T, S, Portable> {
        Builder::<T, S, Portable>::with_capacity_and_hasher(capacity, hasher)
    }

    pub unsafe fn get_index_unchecked<Q>(&self, key: &Q) -> usize
    where
        Q: PortableHash + PortableEq<T> + ?Sized,
    {
        unsafe { self.table.get_index_unchecked(key) }
    }

    pub fn get_index<Q>(&self, key: &Q) -> Option<usize>
    where
        Q: PortableHash + PortableEq<T> + ?Sized,
    {
        self.table.get_index(key)
    }

    pub fn contains<Q>(&self, key: &Q) -> bool
    where
        Q: PortableHash + PortableEq<T> + ?Sized,
    {
        self.table.contains(key)
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&T>
    where
        Q: PortableHash + PortableEq<T> + ?Sized,
    {
        self.table.get(key)
    }

    pub unsafe fn get_unchecked<Q>(&self, key: &Q) -> &T
    where
        Q: PortableHash + PortableEq<T> + ?Sized,
    {
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

impl<T> Builder<T, DefaultHasherSeed, Portable> {
    pub fn new() -> Self {
        Self::with_hasher(DefaultHasherSeed::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, DefaultHasherSeed::new())
    }
}

impl<T, S> Builder<T, S, Portable> {
    pub fn with_hasher(hasher: S) -> Self {
        Self {
            builder: super::TableBuilder::with_hasher(hasher),
        }
    }

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
    pub fn contains<Q>(&self, key: &Q) -> bool
    where
        Q: PortableHash + PortableEq<T> + ?Sized,
    {
        self.builder.contains(key)
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&T>
    where
        Q: PortableHash + PortableEq<T> + ?Sized,
    {
        self.builder.get(key)
    }

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
    pub fn insert(&mut self, value: T) -> bool {
        self.builder.insert(value)
    }

    pub fn replace(&mut self, value: T) -> Option<T> {
        self.builder.replace(value)
    }

    pub fn get_or_insert(&mut self, value: T) -> &T {
        self.builder.get_or_insert(value)
    }

    pub fn take<Q>(&mut self, key: &Q) -> Option<T>
    where
        Q: PortableHash + PortableEq<T> + ?Sized,
    {
        self.builder.take(key)
    }

    pub fn remove<Q>(&mut self, key: &Q) -> bool
    where
        Q: PortableHash + PortableEq<T> + ?Sized,
    {
        self.builder.remove(key)
    }

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
            self.replace(item);
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
            builder.replace(item);
        });
        builder
    }
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
