use super::OrderedSet;
use crate::portability::{OrderedSetOps, Portable};
use crate::table::Table;

use portable::{DefaultHasherSeed, PortableBuildHasher, PortableEq, PortableHash};

use rkyv::{
    Archive, Archived, Deserialize, DeserializeUnsized, Resolver, Serialize, SerializeUnsized,
    bytecheck::CheckBytes,
    rancor::{Fallible, Source},
};

#[doc(inline)]
pub use crate::set_iters::ArchivedIter as Iter;

#[derive(rkyv::Portable, CheckBytes)]
#[bytecheck(crate = rkyv::bytecheck)]
#[repr(transparent)]
pub struct ArchivedOrderedSet<T, S = DefaultHasherSeed>
where
    T: Archive,
    S: Archive,
{
    table: Archived<Table<OrderedSetOps<T>, S, Portable>>,
}

#[repr(transparent)]
pub struct OrderedSetResolver<T, S = DefaultHasherSeed>
where
    T: Archive,
    S: Archive,
{
    table: Resolver<Table<OrderedSetOps<T>, S, Portable>>,
}

impl<T, S> Archive for OrderedSet<T, S, Portable>
where
    T: Archive,
    S: Archive,
{
    type Archived = ArchivedOrderedSet<T, S>;
    type Resolver = OrderedSetResolver<T, S>;

    fn resolve(&self, resolver: Self::Resolver, out: rkyv::Place<Self::Archived>) {
        self.table
            .resolve(resolver.table, unsafe { out.cast_unchecked() });
    }
}

impl<T, S, Ser> Serialize<Ser> for OrderedSet<T, S, Portable>
where
    Ser: Fallible + ?Sized,
    T: Serialize<Ser>,
    S: Serialize<Ser>,
    [T]: SerializeUnsized<Ser>,
    [u32]: SerializeUnsized<Ser>,
{
    fn serialize(&self, serializer: &mut Ser) -> Result<Self::Resolver, Ser::Error> {
        self.table
            .serialize(serializer)
            .map(|table| OrderedSetResolver { table })
    }
}

impl<T, S, D> Deserialize<OrderedSet<T, S, Portable>, D> for ArchivedOrderedSet<T, S>
where
    D: Fallible + ?Sized,
    D::Error: Source,
    T: Archive,
    S: Archive,
    Archived<T>: Deserialize<T, D>,
    Archived<S>: Deserialize<S, D>,
    [Archived<T>]: DeserializeUnsized<[T], D>,
    [Archived<u32>]: DeserializeUnsized<[u32], D>,
{
    fn deserialize(&self, deserializer: &mut D) -> Result<OrderedSet<T, S, Portable>, D::Error> {
        Ok(OrderedSet {
            table: self.table.deserialize(deserializer)?,
        })
    }
}

impl<T, S> ArchivedOrderedSet<T, S>
where
    T: Archive,
    S: Archive,
{
    pub fn hasher(&self) -> &Archived<S> {
        self.table.hasher()
    }

    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            iter: self.table.iter(),
        }
    }

    pub fn as_slice(&self) -> &[Archived<T>] {
        self.table.as_slice()
    }

    pub fn index(&self, index: usize) -> Option<&Archived<T>> {
        self.table.index(index)
    }

    pub unsafe fn index_unchecked(&self, index: usize) -> &Archived<T> {
        unsafe { self.table.index_unchecked(index) }
    }
}

impl<T, S> ArchivedOrderedSet<T, S>
where
    T: Archive + PortableEq<Archived<T>>,
    S: Archive,
    Archived<S>: PortableBuildHasher,
{
    pub unsafe fn get_index_unchecked<Q>(&self, key: &Q) -> usize
    where
        Q: PortableHash + PortableEq<Archived<T>> + ?Sized,
    {
        unsafe { self.table.get_index_unchecked(key) }
    }

    pub fn get_index<Q>(&self, key: &Q) -> Option<usize>
    where
        Q: PortableHash + PortableEq<Archived<T>> + ?Sized,
    {
        self.table.get_index(key)
    }

    pub fn contains<Q>(&self, key: &Q) -> bool
    where
        Q: PortableHash + PortableEq<Archived<T>> + ?Sized,
    {
        self.table.contains(key)
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&Archived<T>>
    where
        Q: PortableHash + PortableEq<Archived<T>> + ?Sized,
    {
        self.table.get(key)
    }

    pub unsafe fn get_unchecked<Q>(&self, key: &Q) -> &Archived<T>
    where
        Q: PortableHash + PortableEq<Archived<T>> + ?Sized,
    {
        unsafe { self.table.get_unchecked(key) }
    }
}

impl<T, S> core::fmt::Debug for ArchivedOrderedSet<T, S>
where
    T: Archive,
    S: Archive,
    Archived<T>: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl<'a, T, S> IntoIterator for &'a ArchivedOrderedSet<T, S>
where
    T: Archive,
    S: Archive,
{
    type Item = &'a Archived<T>;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(feature = "serde")]
impl<T, S> serde::Serialize for ArchivedOrderedSet<T, S>
where
    T: Archive,
    S: Archive,
    Archived<T>: serde::Serialize,
    Archived<S>: serde::Serialize,
{
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: serde::Serializer,
    {
        self.table.serialize_set(serializer)
    }
}
