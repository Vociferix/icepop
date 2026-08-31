use super::{Builder, Map};
use crate::portability::NonPortable;

use equivalent::Equivalent;
use portable::DefaultHasherSeed;

use core::hash::{BuildHasher, Hash};

impl<K, V> Map<K, V, DefaultHasherSeed, NonPortable> {
    pub fn builder() -> Builder<K, V, DefaultHasherSeed, NonPortable> {
        Builder::<K, V, DefaultHasherSeed, NonPortable>::new()
    }

    pub fn builder_with_capacity(capacity: usize) -> Builder<K, V, DefaultHasherSeed, NonPortable> {
        Builder::<K, V, DefaultHasherSeed, NonPortable>::with_capacity(capacity)
    }
}

impl<K, V, S> Map<K, V, S, NonPortable>
where
    S: BuildHasher,
{
    pub fn builder_with_hasher(hasher: S) -> Builder<K, V, S, NonPortable> {
        Builder::<K, V, S, NonPortable>::with_hasher(hasher)
    }

    pub fn builder_with_capacity_and_hasher(
        capacity: usize,
        hasher: S,
    ) -> Builder<K, V, S, NonPortable> {
        Builder::<K, V, S, NonPortable>::with_capacity_and_hasher(capacity, hasher)
    }

    pub unsafe fn get_index_unchecked<Q>(&self, key: &Q) -> usize
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        unsafe { self.table.get_index_unchecked(key) }
    }

    pub fn get_index<Q>(&self, key: &Q) -> Option<usize>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.table.get_index(key)
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.table.contains(key)
    }

    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.table.map_get_key_value(key)
    }

    pub fn get_key_value_mut<Q>(&mut self, key: &Q) -> Option<(&K, &mut V)>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.table.map_get_key_value_mut(key)
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.table.map_get(key)
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.table.map_get_mut(key)
    }

    pub unsafe fn get_key_value_unchecked<Q>(&self, key: &Q) -> (&K, &V)
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        unsafe { self.table.map_get_key_value_unchecked(key) }
    }

    pub unsafe fn get_key_value_unchecked_mut<Q>(&mut self, key: &Q) -> (&K, &mut V)
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        unsafe { self.table.map_get_key_value_unchecked_mut(key) }
    }

    pub unsafe fn get_unchecked<Q>(&self, key: &Q) -> &V
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        unsafe { self.table.map_get_unchecked(key) }
    }

    pub unsafe fn get_unchecked_mut<Q>(&mut self, key: &Q) -> &mut V
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        unsafe { self.table.map_get_unchecked_mut(key) }
    }

    pub fn get_disjoint_key_value_mut<Q, const N: usize>(
        &mut self,
        keys: [&Q; N],
    ) -> [Option<(&K, &mut V)>; N]
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.table.map_get_disjoint_key_value_mut(keys)
    }

    pub fn get_disjoint_mut<Q, const N: usize>(&mut self, keys: [&Q; N]) -> [Option<&mut V>; N]
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.table.map_get_disjoint_mut(keys)
    }

    pub unsafe fn get_disjoint_key_value_unchecked_mut<Q, const N: usize>(
        &mut self,
        keys: [&Q; N],
    ) -> [Option<(&K, &mut V)>; N]
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        unsafe { self.table.map_get_disjoint_key_value_unchecked_mut(keys) }
    }

    pub unsafe fn get_disjoint_unchecked_mut<Q, const N: usize>(
        &mut self,
        keys: [&Q; N],
    ) -> [Option<&mut V>; N]
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
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

impl<K, V> Builder<K, V, DefaultHasherSeed, NonPortable> {
    pub fn new() -> Self {
        Self::with_hasher(DefaultHasherSeed::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, DefaultHasherSeed::new())
    }
}

impl<K, V, S> Builder<K, V, S, NonPortable> {
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

impl<K, V, S> Builder<K, V, S, NonPortable>
where
    S: BuildHasher,
{
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.builder.contains(key)
    }

    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.builder.map_get_key_value(key)
    }

    pub fn get_key_value_mut<Q>(&mut self, key: &Q) -> Option<(&K, &mut V)>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.builder.map_get_key_value_mut(key)
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.builder.map_get(key)
    }

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
    pub fn upsert<F>(&mut self, key: K, update: F) -> bool
    where
        F: FnOnce(Option<V>) -> V,
    {
        self.builder.map_upsert(key, update)
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.builder.map_insert(key, value)
    }

    pub fn get_or_insert_with<Q, F>(&mut self, key: &Q, default: F) -> &mut V
    where
        Q: Hash + Equivalent<K> + alloc::borrow::ToOwned<Owned = K> + ?Sized,
        F: FnOnce() -> V,
    {
        self.builder.map_get_or_insert_with(key, default)
    }

    pub fn get_or_insert_default<Q>(&mut self, key: &Q) -> &mut V
    where
        Q: Hash + Equivalent<K> + alloc::borrow::ToOwned<Owned = K> + ?Sized,
        V: Default,
    {
        self.builder.map_get_or_insert_default(key)
    }

    pub fn get_or_insert<Q>(&mut self, key: &Q, default: V) -> &mut V
    where
        Q: Hash + Equivalent<K> + alloc::borrow::ToOwned<Owned = K> + ?Sized,
    {
        self.builder.map_get_or_insert(key, default)
    }

    pub fn remove_key_value<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.builder.take(key)
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.builder.map_remove(key)
    }

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
