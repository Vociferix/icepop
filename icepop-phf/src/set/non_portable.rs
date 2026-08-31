use super::{Builder, Set};
use crate::portability::NonPortable;

use equivalent::Equivalent;
use portable::DefaultHasherSeed;

use core::hash::{BuildHasher, Hash};

impl<T> Set<T, DefaultHasherSeed, NonPortable> {
    pub fn builder() -> Builder<T, DefaultHasherSeed, NonPortable> {
        Builder::<T, DefaultHasherSeed, NonPortable>::new()
    }

    pub fn builder_with_capacity(capacity: usize) -> Builder<T, DefaultHasherSeed, NonPortable> {
        Builder::<T, DefaultHasherSeed, NonPortable>::with_capacity(capacity)
    }
}

impl<T, S> Set<T, S, NonPortable>
where
    S: BuildHasher,
{
    pub fn builder_with_hasher(hasher: S) -> Builder<T, S, NonPortable> {
        Builder::<T, S, NonPortable>::with_hasher(hasher)
    }

    pub fn builder_with_capacity_and_hasher(
        capacity: usize,
        hasher: S,
    ) -> Builder<T, S, NonPortable> {
        Builder::<T, S, NonPortable>::with_capacity_and_hasher(capacity, hasher)
    }

    pub unsafe fn get_index_unchecked<Q>(&self, key: &Q) -> usize
    where
        Q: Hash + Equivalent<T> + ?Sized,
    {
        unsafe { self.table.get_index_unchecked(key) }
    }

    pub fn get_index<Q>(&self, key: &Q) -> Option<usize>
    where
        Q: Hash + Equivalent<T> + ?Sized,
    {
        self.table.get_index(key)
    }

    pub fn contains<Q>(&self, key: &Q) -> bool
    where
        Q: Hash + Equivalent<T> + ?Sized,
    {
        self.table.contains(key)
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&T>
    where
        Q: Hash + Equivalent<T> + ?Sized,
    {
        self.table.get(key)
    }

    pub unsafe fn get_unchecked<Q>(&self, key: &Q) -> &T
    where
        Q: Hash + Equivalent<T> + ?Sized,
    {
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

impl<T> Builder<T, DefaultHasherSeed, NonPortable> {
    pub fn new() -> Self {
        Self::with_hasher(DefaultHasherSeed::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, DefaultHasherSeed::new())
    }
}

impl<T, S> Builder<T, S, NonPortable> {
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

impl<T, S> Builder<T, S, NonPortable>
where
    S: BuildHasher,
{
    pub fn contains<Q>(&self, key: &Q) -> bool
    where
        Q: Hash + Equivalent<T> + ?Sized,
    {
        self.builder.contains(key)
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&T>
    where
        Q: Hash + Equivalent<T> + ?Sized,
    {
        self.builder.get(key)
    }

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
        Q: Hash + Equivalent<T> + ?Sized,
    {
        self.builder.take(key)
    }

    pub fn remove<Q>(&mut self, key: &Q) -> bool
    where
        Q: Hash + Equivalent<T> + ?Sized,
    {
        self.builder.remove(key)
    }

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
            self.replace(item);
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
            builder.replace(item);
        });
        builder
    }
}
