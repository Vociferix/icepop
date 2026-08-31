use crate::table::Table;

#[cfg(feature = "rkyv")]
use crate::table::rkyv::ArchivedTable;

use equivalent::Equivalent;
use portable::{PortableBuildHasher, PortableEq, PortableHash, PortableHasher};

use core::hash::{BuildHasher, Hash, Hasher};
use core::marker::PhantomData;

use alloc::boxed::Box;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NonPortable {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Portable {}

pub trait BuildHasherOps<P>: Sized {
    type Hasher: HasherOps<P>;

    fn build_hasher(&self) -> Self::Hasher;

    fn hash_one<K>(&self, key: &K) -> u64
    where
        K: HashOps<Self, P> + ?Sized,
    {
        let mut state = self.build_hasher();
        key.hash(&mut state);
        state.finish()
    }
}

pub trait HasherOps<P> {
    fn finish(&self) -> u64;
}

pub trait HashOps<S: BuildHasherOps<P>, P> {
    fn hash(&self, state: &mut S::Hasher);
}

pub trait EqOps<K: ?Sized, P> {
    fn eq(&self, other: &K) -> bool;
}

pub trait TableOps: Sized {
    type Entry: Sized;
    type Key: Sized;
    type Value: Sized;
    type Indices: Sized;

    const HAVE_INDICES: bool;

    fn get_key(entry: &Self::Entry) -> &Self::Key;

    unsafe fn get_index(indices: &Self::Indices, slot: usize) -> usize;

    fn verify<S>(table: &Table<Self, S, Portable>) -> bool;

    #[cfg(feature = "rkyv")]
    fn get_key_archived<'a>(
        entry: &'a rkyv::Archived<Self::Entry>,
    ) -> &'a rkyv::Archived<Self::Key>
    where
        Self::Entry: rkyv::Archive,
        Self::Key: rkyv::Archive,
        Self::Value: rkyv::Archive + 'a;

    #[cfg(feature = "rkyv")]
    unsafe fn get_index_archived(indices: &rkyv::Archived<Self::Indices>, slot: usize) -> usize
    where
        Self::Indices: rkyv::Archive;

    #[cfg(feature = "rkyv")]
    fn verify_archived<S>(table: &ArchivedTable<Self, S>) -> bool
    where
        S: rkyv::Archive,
        Self::Entry: rkyv::Archive,
        Self::Indices: rkyv::Archive;

    #[cfg(all(feature = "rkyv", feature = "serde"))]
    fn archived_indices<S>(table: &ArchivedTable<Self, S>) -> &[rkyv::Archived<u32>]
    where
        S: rkyv::Archive,
        Self::Entry: rkyv::Archive,
        Self::Indices: rkyv::Archive;
}

pub struct OrderedMapOps<K, V>(PhantomData<fn(K, V)>);

pub struct MapOps<K, V>(PhantomData<fn(K, V)>);

pub struct OrderedSetOps<T>(PhantomData<fn(T)>);

pub struct SetOps<T>(PhantomData<fn(T)>);

impl<K, V> TableOps for OrderedMapOps<K, V> {
    type Entry = (K, V);
    type Key = K;
    type Value = V;
    type Indices = Box<[u32]>;

    const HAVE_INDICES: bool = true;

    fn get_key(entry: &Self::Entry) -> &Self::Key {
        &entry.0
    }

    unsafe fn get_index(indices: &Self::Indices, slot: usize) -> usize {
        (unsafe { *indices.get_unchecked(slot) }) as usize
    }

    fn verify<S>(table: &Table<Self, S, Portable>) -> bool {
        let len = table.params.len();
        len == table.indices.len()
            && len == table.entries.len()
            && table.indices.iter().all(|&idx| (idx as usize) < len)
    }

    #[cfg(feature = "rkyv")]
    fn get_key_archived<'a>(entry: &'a rkyv::Archived<Self::Entry>) -> &'a rkyv::Archived<Self::Key>
    where
        Self::Entry: rkyv::Archive,
        Self::Key: rkyv::Archive,
        Self::Value: rkyv::Archive + 'a,
    {
        let entry: &'a rkyv::tuple::ArchivedTuple2<
            rkyv::Archived<Self::Key>,
            rkyv::Archived<Self::Value>,
        > = unsafe { core::mem::transmute(entry) };
        &entry.0
    }

    #[cfg(feature = "rkyv")]
    unsafe fn get_index_archived(indices: &rkyv::Archived<Self::Indices>, slot: usize) -> usize
    where
        Self::Indices: rkyv::Archive,
    {
        (unsafe { *indices.get_unchecked(slot) }).to_native() as usize
    }

    #[cfg(feature = "rkyv")]
    fn verify_archived<S>(table: &ArchivedTable<Self, S>) -> bool
    where
        S: rkyv::Archive,
        Self::Entry: rkyv::Archive,
        Self::Indices: rkyv::Archive,
    {
        let len = table.params.len();
        len == table.indices.len()
            && len == table.entries.len()
            && table
                .indices
                .iter()
                .all(|&idx| (idx.to_native() as usize) < len)
    }

    #[cfg(all(feature = "rkyv", feature = "serde"))]
    fn archived_indices<S>(table: &ArchivedTable<Self, S>) -> &[rkyv::Archived<u32>]
    where
        S: rkyv::Archive,
        Self::Entry: rkyv::Archive,
        Self::Indices: rkyv::Archive,
    {
        &table.indices
    }
}

impl<K, V> TableOps for MapOps<K, V> {
    type Entry = (K, V);
    type Key = K;
    type Value = V;
    type Indices = ();

    const HAVE_INDICES: bool = false;

    fn get_key(entry: &Self::Entry) -> &Self::Key {
        &entry.0
    }

    unsafe fn get_index(_: &Self::Indices, slot: usize) -> usize {
        slot
    }

    fn verify<S>(table: &Table<Self, S, Portable>) -> bool {
        table.params.len() == table.entries.len()
    }

    #[cfg(feature = "rkyv")]
    fn get_key_archived<'a>(entry: &'a rkyv::Archived<Self::Entry>) -> &'a rkyv::Archived<Self::Key>
    where
        Self::Entry: rkyv::Archive,
        Self::Key: rkyv::Archive,
        Self::Value: rkyv::Archive + 'a,
    {
        let entry: &'a rkyv::tuple::ArchivedTuple2<
            rkyv::Archived<Self::Key>,
            rkyv::Archived<Self::Value>,
        > = unsafe { core::mem::transmute(entry) };
        &entry.0
    }

    #[cfg(feature = "rkyv")]
    unsafe fn get_index_archived(_: &rkyv::Archived<Self::Indices>, slot: usize) -> usize
    where
        Self::Indices: rkyv::Archive,
    {
        slot
    }

    #[cfg(feature = "rkyv")]
    fn verify_archived<S>(table: &ArchivedTable<Self, S>) -> bool
    where
        S: rkyv::Archive,
        Self::Entry: rkyv::Archive,
        Self::Indices: rkyv::Archive,
    {
        table.params.len() == table.entries.len()
    }

    #[cfg(all(feature = "rkyv", feature = "serde"))]
    fn archived_indices<S>(table: &ArchivedTable<Self, S>) -> &[rkyv::Archived<u32>]
    where
        S: rkyv::Archive,
        Self::Entry: rkyv::Archive,
        Self::Indices: rkyv::Archive,
    {
        let _ = table;
        &[]
    }
}

impl<T> TableOps for OrderedSetOps<T> {
    type Entry = T;
    type Key = T;
    type Value = ();
    type Indices = Box<[u32]>;

    const HAVE_INDICES: bool = true;

    fn get_key(entry: &Self::Entry) -> &Self::Key {
        entry
    }

    unsafe fn get_index(indices: &Self::Indices, slot: usize) -> usize {
        (unsafe { *indices.get_unchecked(slot) }) as usize
    }

    fn verify<S>(table: &Table<Self, S, Portable>) -> bool {
        let len = table.params.len();
        len == table.indices.len()
            && len == table.entries.len()
            && table.indices.iter().all(|&idx| (idx as usize) < len)
    }

    #[cfg(feature = "rkyv")]
    fn get_key_archived<'a>(entry: &'a rkyv::Archived<Self::Entry>) -> &'a rkyv::Archived<Self::Key>
    where
        Self::Entry: rkyv::Archive,
        Self::Key: rkyv::Archive,
        Self::Value: rkyv::Archive + 'a,
    {
        entry
    }

    #[cfg(feature = "rkyv")]
    unsafe fn get_index_archived(indices: &rkyv::Archived<Self::Indices>, slot: usize) -> usize
    where
        Self::Indices: rkyv::Archive,
    {
        (unsafe { *indices.get_unchecked(slot) }).to_native() as usize
    }

    #[cfg(feature = "rkyv")]
    fn verify_archived<S>(table: &ArchivedTable<Self, S>) -> bool
    where
        S: rkyv::Archive,
        Self::Entry: rkyv::Archive,
        Self::Indices: rkyv::Archive,
    {
        let len = table.params.len();
        len == table.indices.len()
            && len == table.entries.len()
            && table
                .indices
                .iter()
                .all(|&idx| (idx.to_native() as usize) < len)
    }

    #[cfg(all(feature = "rkyv", feature = "serde"))]
    fn archived_indices<S>(table: &ArchivedTable<Self, S>) -> &[rkyv::Archived<u32>]
    where
        S: rkyv::Archive,
        Self::Entry: rkyv::Archive,
        Self::Indices: rkyv::Archive,
    {
        &table.indices
    }
}

impl<T> TableOps for SetOps<T> {
    type Entry = T;
    type Key = T;
    type Value = ();
    type Indices = ();

    const HAVE_INDICES: bool = false;

    fn get_key(entry: &Self::Entry) -> &Self::Key {
        entry
    }

    unsafe fn get_index(_: &Self::Indices, slot: usize) -> usize {
        slot
    }

    fn verify<S>(table: &Table<Self, S, Portable>) -> bool {
        table.params.len() == table.entries.len()
    }

    #[cfg(feature = "rkyv")]
    fn get_key_archived<'a>(entry: &'a rkyv::Archived<Self::Entry>) -> &'a rkyv::Archived<Self::Key>
    where
        Self::Entry: rkyv::Archive,
        Self::Key: rkyv::Archive,
        Self::Value: rkyv::Archive + 'a,
    {
        entry
    }

    #[cfg(feature = "rkyv")]
    unsafe fn get_index_archived(_: &rkyv::Archived<Self::Indices>, slot: usize) -> usize
    where
        Self::Indices: rkyv::Archive,
    {
        slot
    }

    #[cfg(feature = "rkyv")]
    fn verify_archived<S>(table: &ArchivedTable<Self, S>) -> bool
    where
        S: rkyv::Archive,
        Self::Entry: rkyv::Archive,
        Self::Indices: rkyv::Archive,
    {
        table.params.len() == table.entries.len()
    }

    #[cfg(all(feature = "rkyv", feature = "serde"))]
    fn archived_indices<S>(table: &ArchivedTable<Self, S>) -> &[rkyv::Archived<u32>]
    where
        S: rkyv::Archive,
        Self::Entry: rkyv::Archive,
        Self::Indices: rkyv::Archive,
    {
        let _ = table;
        &[]
    }
}

impl<S> BuildHasherOps<NonPortable> for S
where
    S: BuildHasher,
{
    type Hasher = <S as BuildHasher>::Hasher;

    fn build_hasher(&self) -> Self::Hasher {
        <S as BuildHasher>::build_hasher(self)
    }
}

impl<H> HasherOps<NonPortable> for H
where
    H: Hasher,
{
    fn finish(&self) -> u64 {
        <H as Hasher>::finish(self)
    }
}

impl<K, S> HashOps<S, NonPortable> for K
where
    K: Hash + ?Sized,
    S: BuildHasher,
{
    fn hash(&self, state: &mut S::Hasher) {
        <K as Hash>::hash(self, state);
    }
}

impl<T, U> EqOps<U, NonPortable> for T
where
    T: Equivalent<U> + ?Sized,
{
    fn eq(&self, other: &U) -> bool {
        self.equivalent(other)
    }
}

impl<S> BuildHasherOps<Portable> for S
where
    S: PortableBuildHasher,
{
    type Hasher = <S as BuildHasher>::Hasher;

    fn build_hasher(&self) -> Self::Hasher {
        <S as BuildHasher>::build_hasher(self)
    }
}

impl<H> HasherOps<Portable> for H
where
    H: PortableHasher,
{
    fn finish(&self) -> u64 {
        <H as Hasher>::finish(self)
    }
}

impl<K, S> HashOps<S, Portable> for K
where
    K: PortableHash + ?Sized,
    S: PortableBuildHasher,
{
    fn hash(&self, state: &mut S::Hasher) {
        <K as PortableHash>::portable_hash(self, state);
    }
}

impl<T, U> EqOps<U, Portable> for T
where
    T: PortableEq<U> + ?Sized,
{
    fn eq(&self, other: &U) -> bool {
        self.portable_eq(other)
    }
}
