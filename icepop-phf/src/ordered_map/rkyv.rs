use super::OrderedMap;
use crate::portability::{OrderedMapOps, Portable};
use crate::table::{Table, rkyv::ArchivedTable};

use portable::{DefaultHasherSeed, PortableBuildHasher, PortableEq, PortableHash};

use rkyv::{
    Archive, Archived, Deserialize, DeserializeUnsized, Resolver, Serialize, SerializeUnsized,
    bytecheck::CheckBytes,
    rancor::{Fallible, Source},
    seal::Seal,
};

#[doc(inline)]
pub use crate::map_iters::{
    ArchivedIter as Iter, ArchivedIterSeal as IterSeal, ArchivedKeys as Keys,
    ArchivedValues as Values, ArchivedValuesSeal as ValuesSeal,
};

#[derive(rkyv::Portable, CheckBytes)]
#[bytecheck(crate = rkyv::bytecheck)]
#[repr(transparent)]
pub struct ArchivedOrderedMap<K, V, S = DefaultHasherSeed>
where
    K: Archive,
    V: Archive,
    S: Archive,
{
    table: Archived<Table<OrderedMapOps<K, V>, S, Portable>>,
}

#[repr(transparent)]
pub struct OrderedMapResolver<K, V, S = DefaultHasherSeed>
where
    K: Archive,
    V: Archive,
    S: Archive,
{
    table: Resolver<Table<OrderedMapOps<K, V>, S, Portable>>,
}

impl<K, V, S> Archive for OrderedMap<K, V, S, Portable>
where
    K: Archive,
    V: Archive,
    S: Archive,
{
    type Archived = ArchivedOrderedMap<K, V, S>;
    type Resolver = OrderedMapResolver<K, V, S>;

    fn resolve(&self, resolver: Self::Resolver, out: rkyv::Place<Self::Archived>) {
        self.table
            .resolve(resolver.table, unsafe { out.cast_unchecked() });
    }
}

impl<K, V, S, Ser> Serialize<Ser> for OrderedMap<K, V, S, Portable>
where
    Ser: Fallible + ?Sized,
    K: Serialize<Ser>,
    V: Serialize<Ser>,
    S: Serialize<Ser>,
    [(K, V)]: SerializeUnsized<Ser>,
    [u32]: SerializeUnsized<Ser>,
{
    fn serialize(&self, serializer: &mut Ser) -> Result<Self::Resolver, Ser::Error> {
        self.table
            .serialize(serializer)
            .map(|table| OrderedMapResolver { table })
    }
}

impl<K, V, S, D> Deserialize<OrderedMap<K, V, S, Portable>, D> for ArchivedOrderedMap<K, V, S>
where
    D: Fallible + ?Sized,
    D::Error: Source,
    K: Archive,
    V: Archive,
    S: Archive,
    Archived<K>: Deserialize<K, D>,
    Archived<V>: Deserialize<V, D>,
    Archived<S>: Deserialize<S, D>,
    [Archived<(K, V)>]: DeserializeUnsized<[(K, V)], D>,
    [Archived<u32>]: DeserializeUnsized<[u32], D>,
{
    fn deserialize(&self, deserializer: &mut D) -> Result<OrderedMap<K, V, S, Portable>, D::Error> {
        Ok(OrderedMap {
            table: self.table.deserialize(deserializer)?,
        })
    }
}

impl<K, V, S> ArchivedOrderedMap<K, V, S>
where
    K: Archive,
    V: Archive,
    S: Archive,
{
    fn table_seal(this: Seal<'_, Self>) -> Seal<'_, ArchivedTable<OrderedMapOps<K, V>, S>> {
        Seal::new(unsafe { &mut Seal::unseal_unchecked(this).table })
    }

    pub fn hasher(&self) -> &Archived<S> {
        self.table.hasher()
    }

    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            iter: self.table.iter(),
        }
    }

    pub fn iter_seal(this: Seal<'_, Self>) -> IterSeal<'_, K, V> {
        IterSeal {
            iter: unsafe { ArchivedTable::iter_mut(Self::table_seal(this)) },
        }
    }

    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys {
            iter: self.table.iter(),
        }
    }

    pub fn values(&self) -> Values<'_, K, V> {
        Values {
            iter: self.table.iter(),
        }
    }

    pub fn values_seal(this: Seal<'_, Self>) -> ValuesSeal<'_, K, V> {
        ValuesSeal {
            iter: unsafe { ArchivedTable::iter_mut(Self::table_seal(this)) },
        }
    }

    pub fn as_slice(&self) -> &[Archived<(K, V)>] {
        self.table.as_slice()
    }

    pub fn index(&self, index: usize) -> Option<(&Archived<K>, &Archived<V>)> {
        self.table.map_index(index)
    }

    pub fn index_seal(
        this: Seal<'_, Self>,
        index: usize,
    ) -> Option<(&Archived<K>, Seal<'_, Archived<V>>)> {
        ArchivedTable::map_index_seal(Self::table_seal(this), index)
    }

    pub unsafe fn index_unchecked(&self, index: usize) -> (&Archived<K>, &Archived<V>) {
        unsafe { self.table.map_index_unchecked(index) }
    }

    pub unsafe fn index_unchecked_seal(
        this: Seal<'_, Self>,
        index: usize,
    ) -> (&Archived<K>, Seal<'_, Archived<V>>) {
        unsafe { ArchivedTable::map_index_unchecked_seal(Self::table_seal(this), index) }
    }
}

impl<K, V, S> ArchivedOrderedMap<K, V, S>
where
    K: Archive + PortableEq<Archived<K>>,
    V: Archive,
    S: Archive,
    Archived<S>: PortableBuildHasher,
{
    pub unsafe fn get_index_unchecked<Q>(&self, key: &Q) -> usize
    where
        Q: PortableHash + PortableEq<Archived<K>> + ?Sized,
    {
        unsafe { self.table.get_index_unchecked(key) }
    }

    pub fn get_index<Q>(&self, key: &Q) -> Option<usize>
    where
        Q: PortableHash + PortableEq<Archived<K>> + ?Sized,
    {
        self.table.get_index(key)
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        Q: PortableHash + PortableEq<Archived<K>> + ?Sized,
    {
        self.table.contains(key)
    }

    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&Archived<K>, &Archived<V>)>
    where
        Q: PortableHash + PortableEq<Archived<K>> + ?Sized,
    {
        self.table.map_get_key_value(key)
    }

    pub fn get_key_value_seal<'a, Q>(
        this: Seal<'a, Self>,
        key: &Q,
    ) -> Option<(&'a Archived<K>, Seal<'a, Archived<V>>)>
    where
        Q: PortableHash + PortableEq<Archived<K>> + ?Sized,
    {
        ArchivedTable::map_get_key_value_seal(Self::table_seal(this), key)
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&Archived<V>>
    where
        Q: PortableHash + PortableEq<Archived<K>> + ?Sized,
    {
        self.table.map_get(key)
    }

    pub fn get_seal<'a, Q>(this: Seal<'a, Self>, key: &Q) -> Option<Seal<'a, Archived<V>>>
    where
        Q: PortableHash + PortableEq<Archived<K>> + ?Sized,
    {
        ArchivedTable::map_get_seal(Self::table_seal(this), key)
    }

    pub unsafe fn get_key_value_unchecked<Q>(&self, key: &Q) -> (&Archived<K>, &Archived<V>)
    where
        Q: PortableHash + PortableEq<Archived<K>> + ?Sized,
    {
        unsafe { self.table.map_get_key_value_unchecked(key) }
    }

    pub unsafe fn get_key_value_unchecked_seal<'a, Q>(
        this: Seal<'a, Self>,
        key: &Q,
    ) -> (&'a Archived<K>, Seal<'a, Archived<V>>)
    where
        Q: PortableHash + PortableEq<Archived<K>> + ?Sized,
    {
        unsafe { ArchivedTable::map_get_key_value_unchecked_seal(Self::table_seal(this), key) }
    }

    pub unsafe fn get_unchecked<Q>(&self, key: &Q) -> &Archived<V>
    where
        Q: PortableHash + PortableEq<Archived<K>> + ?Sized,
    {
        unsafe { self.table.map_get_unchecked(key) }
    }

    pub unsafe fn get_unchecked_seal<'a, Q>(this: Seal<'a, Self>, key: &Q) -> Seal<'a, Archived<V>>
    where
        Q: PortableHash + PortableEq<Archived<K>> + ?Sized,
    {
        unsafe { ArchivedTable::map_get_unchecked_seal(Self::table_seal(this), key) }
    }

    pub fn get_disjoint_key_value_seal<'a, Q, const N: usize>(
        this: Seal<'a, Self>,
        keys: [&Q; N],
    ) -> [Option<(&'a Archived<K>, Seal<'a, Archived<V>>)>; N]
    where
        Q: PortableHash + PortableEq<Archived<K>> + ?Sized,
    {
        ArchivedTable::map_get_disjoint_key_value_seal(Self::table_seal(this), keys)
    }

    pub fn get_disjoint_seal<'a, Q, const N: usize>(
        this: Seal<'a, Self>,
        keys: [&Q; N],
    ) -> [Option<Seal<'a, Archived<V>>>; N]
    where
        Q: PortableHash + PortableEq<Archived<K>> + ?Sized,
    {
        ArchivedTable::map_get_disjoint_seal(Self::table_seal(this), keys)
    }

    pub unsafe fn get_disjoint_key_value_unchecked_seal<'a, Q, const N: usize>(
        this: Seal<'a, Self>,
        keys: [&Q; N],
    ) -> [Option<(&'a Archived<K>, Seal<'a, Archived<V>>)>; N]
    where
        Q: PortableHash + PortableEq<Archived<K>> + ?Sized,
    {
        unsafe {
            ArchivedTable::map_get_disjoint_key_value_unchecked_seal(Self::table_seal(this), keys)
        }
    }

    pub unsafe fn get_disjoint_unchecked_seal<'a, Q, const N: usize>(
        this: Seal<'a, Self>,
        keys: [&Q; N],
    ) -> [Option<Seal<'a, Archived<V>>>; N]
    where
        Q: PortableHash + PortableEq<Archived<K>> + ?Sized,
    {
        unsafe { ArchivedTable::map_get_disjoint_unchecked_seal(Self::table_seal(this), keys) }
    }
}

impl<K, V, S> core::fmt::Debug for ArchivedOrderedMap<K, V, S>
where
    K: Archive,
    V: Archive,
    S: Archive,
    Archived<K>: core::fmt::Debug,
    Archived<V>: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<'a, K, V, S> IntoIterator for &'a ArchivedOrderedMap<K, V, S>
where
    K: Archive,
    V: Archive,
    S: Archive,
{
    type Item = (&'a Archived<K>, &'a Archived<V>);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(feature = "serde")]
impl<K, V, S> serde::Serialize for ArchivedOrderedMap<K, V, S>
where
    K: Archive,
    V: Archive,
    S: Archive,
    Archived<K>: serde::Serialize,
    Archived<V>: serde::Serialize,
    Archived<S>: serde::Serialize,
{
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: serde::Serializer,
    {
        self.table.serialize_map(serializer)
    }
}
