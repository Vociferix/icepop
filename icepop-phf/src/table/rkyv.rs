//! The archived table, and the read interface the collections expose over it.
//!
//! Mirrors [`Table`](super::Table) field for field, in a `#[repr(C)]` layout with the fields
//! ordered so that the two variable-length arrays come first. Lookup is the same two-round
//! scheme, run against the archived hasher and archived keys.
//!
//! Mutation goes through [`rkyv::seal::Seal`], which is what keeps in-place edits from breaking
//! the table: everything reachable from a sealed archive is a value, never a key.

use super::{Table, unique_indices};
use crate::portability::{Portable, TableOps};

use portable::{DefaultHasherSeed, PortableBuildHasher, PortableEq, PortableHash};

use rkyv::{
    Archive, Archived, Deserialize, Resolver, Serialize,
    bytecheck::{CheckBytes, Verify},
    rancor::{Fallible, Source, fail},
};

use alloc::boxed::Box;

use core::hash::{BuildHasher, Hasher};
use core::marker::PhantomData;

/// A table read in place, out of a serialized buffer.
///
/// The same invariants as [`Table`](super::Table), but they arrive from untrusted bytes rather
/// than from the builder, so the [`Verify`] implementation re-checks them before any unchecked
/// access is allowed.
#[derive(rkyv::Portable, CheckBytes)]
#[bytecheck(crate = rkyv::bytecheck)]
#[bytecheck(verify)]
#[repr(C)]
pub struct ArchivedTable<O: TableOps, S = DefaultHasherSeed>
where
    O::Entry: Archive,
    O::Indices: Archive,
    S: Archive,
{
    pub(crate) params: Archived<Box<[u32]>>,
    pub(crate) indices: Archived<O::Indices>,
    pub(crate) entries: Archived<Box<[O::Entry]>>,
    pub(crate) hasher_builder: Archived<S>,
    pub(crate) global_param: u8,
}

/// Offsets of the out-of-line pieces, filled in by `serialize` and consumed by `resolve`.
pub struct TableResolver<O: TableOps, S = DefaultHasherSeed>
where
    O::Entry: Archive,
    O::Indices: Archive,
    S: Archive,
{
    params: Resolver<Box<[u8]>>,
    indices: Resolver<O::Indices>,
    entries: Resolver<Box<[O::Entry]>>,
    hasher_builder: Resolver<S>,
    global_param: Resolver<u8>,
}

unsafe impl<C, O, S> Verify<C> for ArchivedTable<O, S>
where
    C: Fallible + ?Sized,
    C::Error: Source,
    O: TableOps,
    O::Entry: Archive,
    O::Indices: Archive,
    S: Archive,
{
    /// Rejects an archive whose arrays disagree in length or whose indices point out of bounds.
    fn verify(&self, _: &mut C) -> Result<(), C::Error> {
        #[derive(Debug)]
        struct LayoutError;

        impl core::fmt::Display for LayoutError {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("invalid archive layout for phf table")
            }
        }

        impl core::error::Error for LayoutError {}

        if !O::verify_archived(self) {
            fail!(LayoutError);
        }

        Ok(())
    }
}

impl<O: TableOps, S> Archive for Table<O, S, Portable>
where
    O::Entry: Archive,
    O::Indices: Archive,
    S: Archive,
{
    type Archived = ArchivedTable<O, S>;
    type Resolver = TableResolver<O, S>;

    fn resolve(&self, resolver: Self::Resolver, out: rkyv::Place<Self::Archived>) {
        use rkyv::Place;

        let TableResolver {
            params: params_res,
            indices: indices_res,
            entries: entries_res,
            hasher_builder: hasher_builder_res,
            global_param: global_param_res,
        } = resolver;

        let params_ptr = unsafe { &raw mut (*out.ptr()).params };
        let params = unsafe { Place::from_field_unchecked(out, params_ptr) };
        self.params.resolve(params_res, params);

        let indices_ptr = unsafe { &raw mut (*out.ptr()).indices };
        let indices = unsafe { Place::from_field_unchecked(out, indices_ptr) };
        self.indices.resolve(indices_res, indices);

        let entries_ptr = unsafe { &raw mut (*out.ptr()).entries };
        let entries = unsafe { Place::from_field_unchecked(out, entries_ptr) };
        self.entries.resolve(entries_res, entries);

        let hasher_builder_ptr = unsafe { &raw mut (*out.ptr()).hasher_builder };
        let hasher_builder = unsafe { Place::from_field_unchecked(out, hasher_builder_ptr) };
        self.hasher_builder
            .resolve(hasher_builder_res, hasher_builder);

        let global_param_ptr = unsafe { &raw mut (*out.ptr()).global_param };
        let global_param = unsafe { Place::from_field_unchecked(out, global_param_ptr) };
        self.global_param.resolve(global_param_res, global_param);
    }
}

impl<O: TableOps, S, Ser> Serialize<Ser> for Table<O, S, Portable>
where
    Ser: Fallible + ?Sized,
    O::Entry: Serialize<Ser>,
    O::Indices: Serialize<Ser>,
    S: Serialize<Ser>,
    Box<[u32]>: Serialize<Ser>,
    Box<[O::Entry]>: Serialize<Ser>,
{
    fn serialize(&self, serializer: &mut Ser) -> Result<Self::Resolver, Ser::Error> {
        let params = self.params.serialize(serializer)?;
        let indices = self.indices.serialize(serializer)?;
        let entries = self.entries.serialize(serializer)?;
        let hasher_builder = self.hasher_builder.serialize(serializer)?;
        self.global_param.serialize(serializer)?;

        Ok(TableResolver {
            params,
            indices,
            entries,
            hasher_builder,
            global_param: (),
        })
    }
}

impl<O: TableOps, S, D> Deserialize<Table<O, S, Portable>, D> for ArchivedTable<O, S>
where
    D: Fallible + ?Sized,
    O::Entry: Archive,
    O::Indices: Archive,
    S: Archive,
    Archived<O::Entry>: Deserialize<O::Entry, D>,
    Archived<O::Indices>: Deserialize<O::Indices, D>,
    Archived<S>: Deserialize<S, D>,
    Archived<Box<[u32]>>: Deserialize<Box<[u32]>, D>,
    Archived<Box<[O::Entry]>>: Deserialize<Box<[O::Entry]>, D>,
{
    fn deserialize(&self, deserializer: &mut D) -> Result<Table<O, S, Portable>, D::Error> {
        Ok(Table {
            global_param: self.global_param,
            params: self.params.deserialize(deserializer)?,
            indices: self.indices.deserialize(deserializer)?,
            entries: self.entries.deserialize(deserializer)?,
            hasher_builder: self.hasher_builder.deserialize(deserializer)?,
            _portable: PhantomData,
        })
    }
}

impl<O, S> ArchivedTable<O, S>
where
    O: TableOps,
    O::Entry: Archive,
    O::Key: Archive,
    O::Value: Archive,
    O::Indices: Archive,
    S: Archive,
{
    pub fn hasher(&self) -> &Archived<S> {
        &self.hasher_builder
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn iter(&self) -> core::slice::Iter<'_, Archived<O::Entry>> {
        self.entries.iter()
    }

    /// Iterates the archived entries as plain mutable references.
    ///
    /// # Safety
    ///
    /// Keys must not be modified: a key that no longer hashes to its slot makes the table unable
    /// to find its own entries. The sealed iterators are the safe way to reach values.
    pub unsafe fn iter_mut(
        this: rkyv::seal::Seal<'_, Self>,
    ) -> core::slice::IterMut<'_, Archived<O::Entry>> {
        unsafe { Self::entries_seal(this) }.iter_mut()
    }

    pub fn as_slice(&self) -> &[Archived<O::Entry>] {
        &self.entries
    }

    /// Unseals the entry array.
    ///
    /// # Safety
    ///
    /// The returned slice exposes whole entries, keys included. Nothing may be written through
    /// it but values: a key that no longer hashes to its slot leaves the table unable to find
    /// its own entries. Callers that hand any of this out publicly must reduce it to values
    /// first, which is what the `*_seal` methods do.
    unsafe fn entries_seal(this: rkyv::seal::Seal<'_, Self>) -> &mut [Archived<O::Entry>] {
        let table = unsafe { this.unseal_unchecked() };
        let entries = rkyv::seal::Seal::new(&mut table.entries);
        let slice = rkyv::boxed::ArchivedBox::get_seal(entries);
        unsafe { slice.unseal_unchecked() }
    }

    pub fn index(&self, index: usize) -> Option<&Archived<O::Entry>> {
        self.entries.get().get(index)
    }

    pub fn index_seal(
        this: rkyv::seal::Seal<'_, Self>,
        index: usize,
    ) -> Option<rkyv::seal::Seal<'_, Archived<O::Entry>>> {
        let entries = unsafe { Self::entries_seal(this) };
        entries.get_mut(index).map(rkyv::seal::Seal::new)
    }

    /// # Safety
    ///
    /// `index` must be less than [`len`](Self::len).
    pub unsafe fn index_unchecked(&self, index: usize) -> &Archived<O::Entry> {
        unsafe { self.entries.get_unchecked(index) }
    }

    /// # Safety
    ///
    /// `index` must be less than [`len`](Self::len).
    pub unsafe fn index_unchecked_seal(
        this: rkyv::seal::Seal<'_, Self>,
        index: usize,
    ) -> rkyv::seal::Seal<'_, Archived<O::Entry>> {
        let entries = unsafe { Self::entries_seal(this) };
        rkyv::seal::Seal::new(unsafe { entries.get_unchecked_mut(index) })
    }
}

impl<O, S> ArchivedTable<O, S>
where
    O: TableOps,
    O::Entry: Archive,
    O::Key: Archive,
    O::Value: Archive,
    O::Indices: Archive,
    S: Archive,
    S::Archived: PortableBuildHasher,
{
    /// Runs the two hash rounds and returns the entry index the slot resolves to.
    ///
    /// # Safety
    ///
    /// The table must not be empty. An absent key yields an arbitrary in-bounds index.
    pub unsafe fn get_index_unchecked<Q>(&self, key: &Q) -> usize
    where
        Q: PortableHash + PortableEq<Archived<O::Key>> + ?Sized,
    {
        let modulus = self.entries.len() as u64;
        unsafe {
            core::hint::assert_unchecked(modulus != 0);
        }

        let mut hasher = BuildHasher::build_hasher(&self.hasher_builder);

        self.global_param.portable_hash(&mut hasher);
        key.portable_hash(&mut hasher);
        let param_idx = (Hasher::finish(&hasher) % modulus) as usize;
        let param = unsafe { *self.params.get_unchecked(param_idx) };

        param.portable_hash(&mut hasher);
        let index_idx = (Hasher::finish(&hasher) % modulus) as usize;
        unsafe { O::get_index_archived(&self.indices, index_idx) }
    }

    pub fn get_index<Q>(&self, key: &Q) -> Option<usize>
    where
        Q: PortableHash + PortableEq<Archived<O::Key>> + ?Sized,
    {
        if self.entries.is_empty() {
            return None;
        }

        let index = unsafe { self.get_index_unchecked(key) };
        let entry = unsafe { self.entries.get_unchecked(index) };
        key.portable_eq(O::get_key_archived(entry)).then_some(index)
    }

    pub fn contains<Q>(&self, key: &Q) -> bool
    where
        Q: PortableHash + PortableEq<Archived<O::Key>> + ?Sized,
    {
        if self.entries.is_empty() {
            return false;
        }

        let index = unsafe { self.get_index_unchecked(key) };
        let entry = unsafe { self.entries.get_unchecked(index) };
        key.portable_eq(O::get_key_archived(entry))
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&Archived<O::Entry>>
    where
        Q: PortableHash + PortableEq<Archived<O::Key>> + ?Sized,
    {
        if self.entries.is_empty() {
            return None;
        }

        let index = unsafe { self.get_index_unchecked(key) };
        let entry = unsafe { self.entries.get_unchecked(index) };
        key.portable_eq(O::get_key_archived(entry)).then_some(entry)
    }

    pub fn get_seal<'a, Q>(
        this: rkyv::seal::Seal<'a, Self>,
        key: &Q,
    ) -> Option<rkyv::seal::Seal<'a, Archived<O::Entry>>>
    where
        Q: PortableHash + PortableEq<Archived<O::Key>> + ?Sized,
    {
        if this.entries.is_empty() {
            return None;
        }

        let index = unsafe { this.get_index_unchecked(key) };
        let entry = unsafe { Self::entries_seal(this).get_unchecked_mut(index) };
        key.portable_eq(O::get_key_archived(entry))
            .then_some(rkyv::seal::Seal::new(entry))
    }

    /// # Safety
    ///
    /// The table must not be empty. An absent key yields an arbitrary entry.
    pub unsafe fn get_unchecked<Q>(&self, key: &Q) -> &Archived<O::Entry>
    where
        Q: PortableHash + PortableEq<Archived<O::Key>> + ?Sized,
    {
        let idx = unsafe { self.get_index_unchecked(key) };
        unsafe { self.entries.get_unchecked(idx) }
    }

    /// # Safety
    ///
    /// The table must not be empty. An absent key yields an arbitrary entry.
    pub unsafe fn get_unchecked_seal<'a, Q>(
        this: rkyv::seal::Seal<'a, Self>,
        key: &Q,
    ) -> rkyv::seal::Seal<'a, Archived<O::Entry>>
    where
        Q: PortableHash + PortableEq<Archived<O::Key>> + ?Sized,
    {
        let idx = unsafe { this.get_index_unchecked(key) };
        let entry = unsafe { Self::entries_seal(this).get_unchecked_mut(idx) };
        rkyv::seal::Seal::new(entry)
    }
}

impl<K, V, O, S> ArchivedTable<O, S>
where
    O: TableOps<Entry = (K, V), Key = K, Value = V>,
    O::Entry: Archive,
    O::Key: Archive,
    O::Value: Archive,
    O::Indices: Archive,
    S: Archive,
{
    pub fn map_index<'a>(&'a self, index: usize) -> Option<(&'a Archived<K>, &'a Archived<V>)> {
        self.index(index).map(|entry| {
            let entry: &'a rkyv::tuple::ArchivedTuple2<Archived<K>, Archived<V>> =
                unsafe { core::mem::transmute(entry) };
            (&entry.0, &entry.1)
        })
    }

    pub fn map_index_seal<'a>(
        this: rkyv::seal::Seal<'a, Self>,
        index: usize,
    ) -> Option<(&'a Archived<K>, rkyv::seal::Seal<'a, Archived<V>>)> {
        Self::index_seal(this, index).map(|entry| {
            let entry = unsafe { entry.unseal_unchecked() };
            let entry: &'a mut rkyv::tuple::ArchivedTuple2<Archived<K>, Archived<V>> =
                unsafe { core::mem::transmute(entry) };
            (&entry.0, rkyv::seal::Seal::new(&mut entry.1))
        })
    }

    /// # Safety
    ///
    /// `index` must be less than [`len`](Self::len).
    pub unsafe fn map_index_unchecked<'a>(
        &'a self,
        index: usize,
    ) -> (&'a Archived<K>, &'a Archived<V>) {
        let entry = unsafe { self.index_unchecked(index) };
        let entry: &'a rkyv::tuple::ArchivedTuple2<Archived<K>, Archived<V>> =
            unsafe { core::mem::transmute(entry) };
        (&entry.0, &entry.1)
    }

    /// # Safety
    ///
    /// `index` must be less than [`len`](Self::len).
    pub unsafe fn map_index_unchecked_seal<'a>(
        this: rkyv::seal::Seal<'a, Self>,
        index: usize,
    ) -> (&'a Archived<K>, rkyv::seal::Seal<'a, Archived<V>>) {
        let entry = unsafe { Self::index_unchecked_seal(this, index) };
        let entry: &'a mut rkyv::tuple::ArchivedTuple2<Archived<K>, Archived<V>> =
            unsafe { core::mem::transmute(entry) };
        (&entry.0, rkyv::seal::Seal::new(&mut entry.1))
    }
}

impl<K, V, O, S> ArchivedTable<O, S>
where
    O: TableOps<Entry = (K, V), Key = K, Value = V>,
    K: Archive,
    V: Archive,
    O::Entry: Archive,
    O::Key: Archive,
    O::Value: Archive,
    O::Indices: Archive,
    S: Archive,
    S::Archived: PortableBuildHasher,
{
    pub fn map_get_key_value<'a, Q>(&'a self, key: &Q) -> Option<(&'a Archived<K>, &'a Archived<V>)>
    where
        Q: PortableHash + PortableEq<Archived<O::Key>> + ?Sized,
    {
        self.get(key).map(|entry| {
            let entry: &'a rkyv::tuple::ArchivedTuple2<Archived<K>, Archived<V>> =
                unsafe { core::mem::transmute(entry) };
            (&entry.0, &entry.1)
        })
    }

    pub fn map_get_key_value_seal<'a, Q>(
        this: rkyv::seal::Seal<'a, Self>,
        key: &Q,
    ) -> Option<(&'a Archived<K>, rkyv::seal::Seal<'a, Archived<V>>)>
    where
        Q: PortableHash + PortableEq<Archived<O::Key>> + ?Sized,
    {
        Self::get_seal(this, key).map(|entry| {
            let entry: &'a mut rkyv::tuple::ArchivedTuple2<Archived<K>, Archived<V>> =
                unsafe { core::mem::transmute(entry) };
            (&entry.0, rkyv::seal::Seal::new(&mut entry.1))
        })
    }

    /// # Safety
    ///
    /// The table must not be empty. An absent key yields an arbitrary entry.
    pub unsafe fn map_get_key_value_unchecked<'a, Q>(
        &'a self,
        key: &Q,
    ) -> (&'a Archived<K>, &'a Archived<V>)
    where
        Q: PortableHash + PortableEq<Archived<O::Key>> + ?Sized,
    {
        let entry = unsafe { self.get_unchecked(key) };
        let entry: &'a rkyv::tuple::ArchivedTuple2<Archived<K>, Archived<V>> =
            unsafe { core::mem::transmute(entry) };
        (&entry.0, &entry.1)
    }

    /// # Safety
    ///
    /// The table must not be empty. An absent key yields an arbitrary entry.
    pub unsafe fn map_get_key_value_unchecked_seal<'a, Q>(
        this: rkyv::seal::Seal<'a, Self>,
        key: &Q,
    ) -> (&'a Archived<K>, rkyv::seal::Seal<'a, Archived<V>>)
    where
        Q: PortableHash + PortableEq<Archived<O::Key>> + ?Sized,
    {
        let entry = unsafe { Self::get_unchecked_seal(this, key) };
        let entry: &'a mut rkyv::tuple::ArchivedTuple2<Archived<K>, Archived<V>> =
            unsafe { core::mem::transmute(entry) };
        (&entry.0, rkyv::seal::Seal::new(&mut entry.1))
    }

    pub fn map_get<'a, Q>(&'a self, key: &Q) -> Option<&'a Archived<V>>
    where
        Q: PortableHash + PortableEq<Archived<O::Key>> + ?Sized,
        K::Archived: 'a,
    {
        self.get(key).map(|entry| {
            let entry: &'a rkyv::tuple::ArchivedTuple2<Archived<K>, Archived<V>> =
                unsafe { core::mem::transmute(entry) };
            &entry.1
        })
    }

    pub fn map_get_seal<'a, Q>(
        this: rkyv::seal::Seal<'a, Self>,
        key: &Q,
    ) -> Option<rkyv::seal::Seal<'a, Archived<V>>>
    where
        Q: PortableHash + PortableEq<Archived<O::Key>> + ?Sized,
        K::Archived: 'a,
    {
        Self::get_seal(this, key).map(|entry| {
            let entry: &'a mut rkyv::tuple::ArchivedTuple2<Archived<K>, Archived<V>> =
                unsafe { core::mem::transmute(entry) };
            rkyv::seal::Seal::new(&mut entry.1)
        })
    }

    /// # Safety
    ///
    /// The table must not be empty. An absent key yields an arbitrary value.
    pub unsafe fn map_get_unchecked<'a, Q>(&'a self, key: &Q) -> &'a Archived<V>
    where
        Q: PortableHash + PortableEq<Archived<O::Key>> + ?Sized,
        K::Archived: 'a,
    {
        let entry = unsafe { self.get_unchecked(key) };
        let entry: &'a rkyv::tuple::ArchivedTuple2<Archived<K>, Archived<V>> =
            unsafe { core::mem::transmute(entry) };
        &entry.1
    }

    /// # Safety
    ///
    /// The table must not be empty. An absent key yields an arbitrary value.
    pub unsafe fn map_get_unchecked_seal<'a, Q>(
        this: rkyv::seal::Seal<'a, Self>,
        key: &Q,
    ) -> rkyv::seal::Seal<'a, Archived<V>>
    where
        Q: PortableHash + PortableEq<Archived<O::Key>> + ?Sized,
        K::Archived: 'a,
    {
        let entry = unsafe { Self::get_unchecked_seal(this, key) };
        let entry: &'a mut rkyv::tuple::ArchivedTuple2<Archived<K>, Archived<V>> =
            unsafe { core::mem::transmute(entry) };
        rkyv::seal::Seal::new(&mut entry.1)
    }

    /// # Panics
    ///
    /// Panics if two keys resolve to the same entry.
    #[allow(clippy::type_complexity)]
    pub fn map_get_disjoint_key_value_seal<'a, Q, const N: usize>(
        this: rkyv::seal::Seal<'a, Self>,
        keys: [&Q; N],
    ) -> [Option<(&'a Archived<K>, rkyv::seal::Seal<'a, Archived<V>>)>; N]
    where
        Q: PortableHash + PortableEq<Archived<O::Key>> + ?Sized,
    {
        let indices = keys.map(|key| this.get_index(key));
        assert!(unique_indices(&indices), "duplicate key found");
        let entries = unsafe { Self::entries_seal(this) };
        indices.map(|idx| {
            idx.map(|idx| {
                let entry: &'a mut rkyv::tuple::ArchivedTuple2<Archived<K>, Archived<V>> =
                    unsafe { core::mem::transmute(&mut *entries.as_mut_ptr().add(idx)) };
                (&entry.0, rkyv::seal::Seal::new(&mut entry.1))
            })
        })
    }

    /// # Panics
    ///
    /// Panics if two keys resolve to the same entry.
    pub fn map_get_disjoint_seal<'a, Q, const N: usize>(
        this: rkyv::seal::Seal<'a, Self>,
        keys: [&Q; N],
    ) -> [Option<rkyv::seal::Seal<'a, Archived<V>>>; N]
    where
        Q: PortableHash + PortableEq<Archived<O::Key>> + ?Sized,
        K::Archived: 'a,
    {
        let indices = keys.map(|key| this.get_index(key));
        assert!(unique_indices(&indices), "duplicate key found");
        let entries = unsafe { Self::entries_seal(this) };
        indices.map(|idx| {
            idx.map(|idx| {
                let entry: &'a mut rkyv::tuple::ArchivedTuple2<Archived<K>, Archived<V>> =
                    unsafe { core::mem::transmute(&mut *entries.as_mut_ptr().add(idx)) };
                rkyv::seal::Seal::new(&mut entry.1)
            })
        })
    }

    /// # Safety
    ///
    /// No two keys may resolve to the same entry; each value is handed out as a distinct seal.
    #[allow(clippy::type_complexity)]
    pub unsafe fn map_get_disjoint_key_value_unchecked_seal<'a, Q, const N: usize>(
        this: rkyv::seal::Seal<'a, Self>,
        keys: [&Q; N],
    ) -> [Option<(&'a Archived<K>, rkyv::seal::Seal<'a, Archived<V>>)>; N]
    where
        Q: PortableHash + PortableEq<Archived<O::Key>> + ?Sized,
    {
        let indices = keys.map(|key| this.get_index(key));
        let entries = unsafe { Self::entries_seal(this) };
        indices.map(|idx| {
            idx.map(|idx| {
                let entry: &'a mut rkyv::tuple::ArchivedTuple2<Archived<K>, Archived<V>> =
                    unsafe { core::mem::transmute(&mut *entries.as_mut_ptr().add(idx)) };
                (&entry.0, rkyv::seal::Seal::new(&mut entry.1))
            })
        })
    }

    /// # Safety
    ///
    /// No two keys may resolve to the same entry; each value is handed out as a distinct seal.
    pub unsafe fn map_get_disjoint_unchecked_seal<'a, Q, const N: usize>(
        this: rkyv::seal::Seal<'a, Self>,
        keys: [&Q; N],
    ) -> [Option<rkyv::seal::Seal<'a, Archived<V>>>; N]
    where
        Q: PortableHash + PortableEq<Archived<O::Key>> + ?Sized,
        K::Archived: 'a,
    {
        let indices = keys.map(|key| this.get_index(key));
        let entries = unsafe { Self::entries_seal(this) };
        indices.map(|idx| {
            idx.map(|idx| {
                let entry: &'a mut rkyv::tuple::ArchivedTuple2<Archived<K>, Archived<V>> =
                    unsafe { core::mem::transmute(&mut *entries.as_mut_ptr().add(idx)) };
                rkyv::seal::Seal::new(&mut entry.1)
            })
        })
    }
}

impl<O, S1, S2> PartialEq<ArchivedTable<O, S2>> for ArchivedTable<O, S1>
where
    O: TableOps,
    O::Entry: Archive,
    O::Key: Archive,
    O::Value: Archive,
    O::Indices: Archive,
    S1: Archive,
    S2: Archive,
    Archived<O::Entry>: PartialEq,
    Archived<O::Key>: PortableHash + PortableEq,
    Archived<S1>: PortableBuildHasher,
    Archived<S2>: PortableBuildHasher,
{
    fn eq(&self, other: &ArchivedTable<O, S2>) -> bool {
        if self.len() != other.len() {
            return false;
        }

        for entry in self.iter() {
            let Some(other) = other.get(O::get_key_archived(entry)) else {
                return false;
            };
            if entry != other {
                return false;
            }
        }

        true
    }
}

impl<O, S> Eq for ArchivedTable<O, S>
where
    O: TableOps,
    O::Entry: Archive,
    O::Key: Archive,
    O::Value: Archive,
    O::Indices: Archive,
    S: Archive,
    Archived<O::Entry>: Eq,
    Archived<O::Key>: PortableHash + PortableEq,
    Archived<S>: PortableBuildHasher,
{
}

cfg_select!(feature = "serde" => {
    impl<O, S> ArchivedTable<O, S>
    where
        O: TableOps,
        O::Entry: Archive,
        O::Key: Archive,
        O::Value: Archive,
        O::Indices: Archive,
        S: Archive,
        Archived<O::Entry>: serde::Serialize,
        Archived<S>: serde::Serialize,
    {
        /// Serializes an archived set in the same shape a live [`Table`](super::Table) produces, so
        /// the two forms are interchangeable to a `serde` consumer.
        pub fn serialize_set<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
        where
            Ser: serde::Serializer,
        {
            use serde::ser::SerializeStruct;

            let mut ser = serializer.serialize_struct("Table", 5 - (if O::HAVE_INDICES { 0 } else { 1 }))?;
            ser.serialize_field("hasher", &self.hasher_builder)?;
            ser.serialize_field("entries", &*self.entries)?;
            ser.serialize_field("global_param", &self.global_param)?;
            ser.serialize_field("params", &SerdeArchivedU32s(&self.params))?;
            if O::HAVE_INDICES {
                ser.serialize_field("indices", &SerdeArchivedU32s(O::archived_indices(self)))?;
            }
            ser.end()
        }
    }

    impl<K, V, O, S> ArchivedTable<O, S>
    where
        O: TableOps<Entry = (K, V), Key = K, Value = V>,
        K: Archive,
        V: Archive,
        O::Indices: Archive,
        S: Archive,
        Archived<K>: serde::Serialize,
        Archived<V>: serde::Serialize,
        Archived<S>: serde::Serialize,
    {
        /// Serializes an archived map in the same shape a live [`Table`](super::Table) produces.
        ///
        /// Archived entries are `ArchivedTuple2`, not tuples, so they are re-projected into pairs on
        /// the way out.
        pub fn serialize_map<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
        where
            Ser: serde::Serializer,
        {
            use serde::ser::SerializeStruct;

            struct Entries<'a, K, V>(&'a [rkyv::tuple::ArchivedTuple2<K, V>]);

            impl<'a, K, V> serde::Serialize for Entries<'a, K, V>
            where
                K: serde::Serialize,
                V: serde::Serialize,
            {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    serializer.collect_seq(self.0.iter().map(|entry| (&entry.0, &entry.1)))
                }
            }

            let mut ser = serializer.serialize_struct("Table", 5 - (if O::HAVE_INDICES { 0 } else { 1 }))?;
            ser.serialize_field("hasher", &self.hasher_builder)?;
            ser.serialize_field("entries", &Entries(&self.entries))?;
            ser.serialize_field("global_param", &self.global_param)?;
            ser.serialize_field("params", &SerdeArchivedU32s(&self.params))?;
            if O::HAVE_INDICES {
                ser.serialize_field("indices", &SerdeArchivedU32s(O::archived_indices(self)))?;
            }
            ser.end()
        }
    }

    struct SerdeArchivedU32s<'a>(&'a [Archived<u32>]);

    impl serde::Serialize for SerdeArchivedU32s<'_> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.collect_seq(self.0.iter().map(|i| i.to_native()))
        }
    }
} _ => {});
