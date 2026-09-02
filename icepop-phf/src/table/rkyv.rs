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
    params: Resolver<Box<[u32]>>,
    indices: Resolver<O::Indices>,
    entries: Resolver<Box<[O::Entry]>>,
    hasher_builder: Resolver<S>,
    global_param: Resolver<u8>,
}

// SAFETY: `verify` checks exactly the invariants the unchecked accessors rely on — that the
// arrays agree in length and that every index is in bounds — and returns an error rather
// than accepting an archive that violates them.
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

        // SAFETY: `out` is a valid place for `Self`, so projecting to its `params` field stays
        // inside that allocation; the pointer is not dereferenced here.
        let params_ptr = unsafe { &raw mut (*out.ptr()).params };
        // SAFETY: `params_ptr` was just projected out of `out`, so it names that field of it.
        let params = unsafe { Place::from_field_unchecked(out, params_ptr) };
        self.params.resolve(params_res, params);

        // SAFETY: `out` is a valid place for `Self`, so projecting to its `indices` field stays
        // inside that allocation; the pointer is not dereferenced here.
        let indices_ptr = unsafe { &raw mut (*out.ptr()).indices };
        // SAFETY: `indices_ptr` was just projected out of `out`, so it names that field of it.
        let indices = unsafe { Place::from_field_unchecked(out, indices_ptr) };
        self.indices.resolve(indices_res, indices);

        // SAFETY: `out` is a valid place for `Self`, so projecting to its `entries` field stays
        // inside that allocation; the pointer is not dereferenced here.
        let entries_ptr = unsafe { &raw mut (*out.ptr()).entries };
        // SAFETY: `entries_ptr` was just projected out of `out`, so it names that field of it.
        let entries = unsafe { Place::from_field_unchecked(out, entries_ptr) };
        self.entries.resolve(entries_res, entries);

        // SAFETY: `out` is a valid place for `Self`, so projecting to its `hasher_builder` field
        // stays inside that allocation; the pointer is not dereferenced here.
        let hasher_builder_ptr = unsafe { &raw mut (*out.ptr()).hasher_builder };
        // SAFETY: `hasher_builder_ptr` was just projected out of `out`, so it names that field of
        // it.
        let hasher_builder = unsafe { Place::from_field_unchecked(out, hasher_builder_ptr) };
        self.hasher_builder
            .resolve(hasher_builder_res, hasher_builder);

        // SAFETY: `out` is a valid place for `Self`, so projecting to its `global_param` field
        // stays inside that allocation; the pointer is not dereferenced here.
        let global_param_ptr = unsafe { &raw mut (*out.ptr()).global_param };
        // SAFETY: `global_param_ptr` was just projected out of `out`, so it names that field of it.
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
        // SAFETY: the caller guarantees keys are left untouched, which is what `entries_seal`
        // requires; this function forwards that same obligation to its own caller.
        unsafe { Self::entries_seal(this) }.iter_mut()
    }

    pub fn as_slice(&self) -> &[Archived<O::Entry>] {
        &self.entries
    }

    /// Unseals the entry array.
    ///
    /// # Safety
    ///
    /// The returned slice exposes whole entries, keys included, as plain references. That
    /// escapes the protection a [`Seal`](rkyv::seal::Seal) gives, under which rkyv's `!Unpin`
    /// archived types cannot be written at all: through this `&mut`, two same-typed fields of
    /// one entry can be swapped, which leaves any relative pointer in them dangling.
    ///
    /// So nothing may be written through the result but values, and even those only in place.
    /// Callers handing this out publicly must narrow it to a seal first, as `index_seal` and
    /// `get_seal` do.
    unsafe fn entries_seal(this: rkyv::seal::Seal<'_, Self>) -> &mut [Archived<O::Entry>] {
        // SAFETY: the caller guarantees nothing but values is written through the result, so
        // unsealing the table cannot invalidate the archive.
        let table = unsafe { this.unseal_unchecked() };
        let entries = rkyv::seal::Seal::new(&mut table.entries);
        let slice = rkyv::boxed::ArchivedBox::get_seal(entries);
        // SAFETY: same obligation, forwarded to the caller of this function.
        unsafe { slice.unseal_unchecked() }
    }

    pub fn index(&self, index: usize) -> Option<&Archived<O::Entry>> {
        self.entries.get().get(index)
    }

    pub fn index_seal(
        this: rkyv::seal::Seal<'_, Self>,
        index: usize,
    ) -> Option<rkyv::seal::Seal<'_, Archived<O::Entry>>> {
        // SAFETY: the `&mut` is immediately narrowed back to a `Seal` over one entry and never
        // used to write, so no key is written through it. What the caller does with that seal
        // cannot break the archive either: rkyv marks every position-dependent archived type
        // `!Unpin`, so `Seal::unseal` refuses to hand out a `&mut` to a key holding a relative
        // pointer, and a key that is plain data can only be given a wrong value.
        let entries = unsafe { Self::entries_seal(this) };
        entries.get_mut(index).map(rkyv::seal::Seal::new)
    }

    /// # Safety
    ///
    /// `index` must be less than [`len`](Self::len).
    pub unsafe fn index_unchecked(&self, index: usize) -> &Archived<O::Entry> {
        // SAFETY: the caller guarantees `index` is within the entry array.
        unsafe { self.entries.get_unchecked(index) }
    }

    /// # Safety
    ///
    /// `index` must be less than [`len`](Self::len).
    pub unsafe fn index_unchecked_seal(
        this: rkyv::seal::Seal<'_, Self>,
        index: usize,
    ) -> rkyv::seal::Seal<'_, Archived<O::Entry>> {
        // SAFETY: the caller guarantees `index` is within the entry array, so only that one entry
        // is sealed and no key is written here.
        let entries = unsafe { Self::entries_seal(this) };
        // SAFETY: the caller guarantees `index` is within the entry array.
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
        // SAFETY: the caller guarantees the archived table is not empty. Stating that here is what
        // lets the two reductions below compile without a division-by-zero check.
        unsafe {
            core::hint::assert_unchecked(modulus != 0);
        }

        let mut hasher = BuildHasher::build_hasher(&self.hasher_builder);

        self.global_param.portable_hash(&mut hasher);
        key.portable_hash(&mut hasher);
        let param_idx = (Hasher::finish(&hasher) % modulus) as usize;
        // SAFETY: `param_idx` is a remainder modulo the entry count, and `params` holds exactly
        // that many elements.
        let param = unsafe { *self.params.get_unchecked(param_idx) };

        param.portable_hash(&mut hasher);
        let index_idx = (Hasher::finish(&hasher) % modulus) as usize;
        // SAFETY: `index_idx` is a remainder modulo the entry count, which is the bound
        // `get_index_archived` requires of a slot.
        unsafe { O::get_index_archived(&self.indices, index_idx) }
    }

    pub fn get_index<Q>(&self, key: &Q) -> Option<usize>
    where
        Q: PortableHash + PortableEq<Archived<O::Key>> + ?Sized,
    {
        if self.entries.is_empty() {
            return None;
        }

        // SAFETY: the early return above establishes that the archived table is not empty.
        let index = unsafe { self.get_index_unchecked(key) };
        // SAFETY: `get_index_unchecked` resolves to an index within the entry array.
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

        // SAFETY: the early return above establishes that the archived table is not empty.
        let index = unsafe { self.get_index_unchecked(key) };
        // SAFETY: `get_index_unchecked` resolves to an index within the entry array.
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

        // SAFETY: the early return above establishes that the archived table is not empty.
        let index = unsafe { self.get_index_unchecked(key) };
        // SAFETY: `get_index_unchecked` resolves to an index within the entry array.
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

        // SAFETY: the early return above establishes that the archived table is not empty.
        let index = unsafe { this.get_index_unchecked(key) };
        // SAFETY: `get_index_unchecked` resolves to an index within the entry array, and the
        // `&mut` is only narrowed back to a `Seal` over that one entry, so no key is written
        // through it. See `index_seal` for why the seal handed on cannot break the archive.
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
        // SAFETY: `get_index_unchecked` has this function's contract is the one the caller upheld.
        let idx = unsafe { self.get_index_unchecked(key) };
        // SAFETY: `get_index_unchecked` resolves to an index within the entry array.
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
        // SAFETY: `get_index_unchecked` has this function's contract is the one the caller upheld.
        let idx = unsafe { this.get_index_unchecked(key) };
        // SAFETY: `get_index_unchecked` resolves to an index within the entry array, and the
        // `&mut` is only narrowed back to a `Seal` over that one entry, so no key is written
        // through it. See `index_seal` for why the seal handed on cannot break the archive.
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
            // SAFETY: `O::Entry` is `(K, V)`, and rkyv archives a pair as `ArchivedTuple2`, so
            // source and target are the same type; the associated type hides that from the
            // compiler.
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
            // SAFETY: the seal covers one entry of this table, and only its value is handed back
            // below, so unsealing it here writes no key.
            let entry = unsafe { entry.unseal_unchecked() };
            // SAFETY: `O::Entry` is `(K, V)`, and rkyv archives a pair as `ArchivedTuple2`, so
            // source and target are the same type; the associated type hides that from the
            // compiler.
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
        // SAFETY: `index_unchecked` has this function's contract, which the caller upheld.
        let entry = unsafe { self.index_unchecked(index) };
        // SAFETY: `O::Entry` is `(K, V)`, and rkyv archives a pair as `ArchivedTuple2`, so source
        // and target are the same type; the associated type hides that from the compiler.
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
        // SAFETY: `index_unchecked_seal` has this function's contract is the one the caller upheld.
        let entry = unsafe { Self::index_unchecked_seal(this, index) };
        // SAFETY: `O::Entry` is `(K, V)`, and rkyv archives a pair as `ArchivedTuple2`, so source
        // and target are the same type; the associated type hides that from the compiler.
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
            // SAFETY: `O::Entry` is `(K, V)`, and rkyv archives a pair as `ArchivedTuple2`, so
            // source and target are the same type; the associated type hides that from the
            // compiler.
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
            // SAFETY: `O::Entry` is `(K, V)`, and rkyv archives a pair as `ArchivedTuple2`, so
            // source and target are the same type; the associated type hides that from the
            // compiler.
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
        // SAFETY: `get_unchecked` has this function's contract, which the caller upheld.
        let entry = unsafe { self.get_unchecked(key) };
        // SAFETY: `O::Entry` is `(K, V)`, and rkyv archives a pair as `ArchivedTuple2`, so source
        // and target are the same type; the associated type hides that from the compiler.
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
        // SAFETY: `get_unchecked_seal` has this function's contract is the one the caller upheld.
        let entry = unsafe { Self::get_unchecked_seal(this, key) };
        // SAFETY: `O::Entry` is `(K, V)`, and rkyv archives a pair as `ArchivedTuple2`, so source
        // and target are the same type; the associated type hides that from the compiler.
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
            // SAFETY: `O::Entry` is `(K, V)`, and rkyv archives a pair as `ArchivedTuple2`, so
            // source and target are the same type; the associated type hides that from the
            // compiler.
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
            // SAFETY: `O::Entry` is `(K, V)`, and rkyv archives a pair as `ArchivedTuple2`, so
            // source and target are the same type; the associated type hides that from the
            // compiler.
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
        // SAFETY: `get_unchecked` has this function's contract, which the caller upheld.
        let entry = unsafe { self.get_unchecked(key) };
        // SAFETY: `O::Entry` is `(K, V)`, and rkyv archives a pair as `ArchivedTuple2`, so source
        // and target are the same type; the associated type hides that from the compiler.
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
        // SAFETY: `get_unchecked_seal` has this function's contract is the one the caller upheld.
        let entry = unsafe { Self::get_unchecked_seal(this, key) };
        // SAFETY: `O::Entry` is `(K, V)`, and rkyv archives a pair as `ArchivedTuple2`, so source
        // and target are the same type; the associated type hides that from the compiler.
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
        // SAFETY: the entry array is only used to build seals over single entries below; no key is
        // written through it here.
        let entries = unsafe { Self::entries_seal(this) };
        indices.map(|idx| {
            idx.map(|idx| {
                // SAFETY: `idx` came from `get_index`, so it is within the entry array, and the
                // assert above rules out two keys sharing one, so no two borrows alias. `O::Entry`
                // is `(K, V)`, and rkyv archives a pair as `ArchivedTuple2`, so source and target
                // are the same type; the associated type hides that from the compiler.
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
        // SAFETY: the entry array is only used to build seals over single entries below; no key is
        // written through it here.
        let entries = unsafe { Self::entries_seal(this) };
        indices.map(|idx| {
            idx.map(|idx| {
                // SAFETY: `idx` came from `get_index`, so it is within the entry array, and the
                // assert above rules out two keys sharing one, so no two borrows alias. `O::Entry`
                // is `(K, V)`, and rkyv archives a pair as `ArchivedTuple2`, so source and target
                // are the same type; the associated type hides that from the compiler.
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
        // SAFETY: the entry array is only used to build seals over single entries below; no key is
        // written through it here.
        let entries = unsafe { Self::entries_seal(this) };
        indices.map(|idx| {
            idx.map(|idx| {
                // SAFETY: `idx` came from `get_index`, so it is within the entry array, and the
                // caller guarantees no two keys share an entry, so no two borrows alias. `O::Entry`
                // is `(K, V)`, and rkyv archives a pair as `ArchivedTuple2`, so source and target
                // are the same type; the associated type hides that from the compiler.
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
        // SAFETY: the entry array is only used to build seals over single entries below; no key is
        // written through it here.
        let entries = unsafe { Self::entries_seal(this) };
        indices.map(|idx| {
            idx.map(|idx| {
                // SAFETY: `idx` came from `get_index`, so it is within the entry array, and the
                // caller guarantees no two keys share an entry, so no two borrows alias. `O::Entry`
                // is `(K, V)`, and rkyv archives a pair as `ArchivedTuple2`, so source and target
                // are the same type; the associated type hides that from the compiler.
                let entry: &'a mut rkyv::tuple::ArchivedTuple2<Archived<K>, Archived<V>> =
                    unsafe { core::mem::transmute(&mut *entries.as_mut_ptr().add(idx)) };
                rkyv::seal::Seal::new(&mut entry.1)
            })
        })
    }
}

// Hand-written for the same reason as `Table`'s: a derive would demand `Debug` of the marker
// type `O`, which carries no data and implements nothing.
impl<O, S> core::fmt::Debug for ArchivedTable<O, S>
where
    O: TableOps,
    O::Entry: Archive,
    O::Indices: Archive,
    S: Archive,
    Archived<O::Entry>: core::fmt::Debug,
    Archived<O::Indices>: core::fmt::Debug,
    Archived<S>: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // The boxed arrays are dereferenced so their contents show rather than the relative
        // pointer that `ArchivedBox` prints. `indices` cannot be: it is `()` when the shape has
        // no slot table, so there is no one type to reach through.
        f.debug_struct("ArchivedTable")
            .field("global_param", &self.global_param)
            .field("params", &&*self.params)
            .field("indices", &self.indices)
            .field("entries", &&*self.entries)
            .field("hasher_builder", &self.hasher_builder)
            .finish()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portability::{MapOps, OrderedSetOps, SetOps};
    use crate::table::Builder;

    use alloc::vec::Vec;
    use rkyv::rancor::Error;
    use rkyv::seal::Seal;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    type SetTable = Table<SetOps<u32>, DefaultHasherSeed, Portable>;
    type MapTable = Table<MapOps<u32, u32>, DefaultHasherSeed, Portable>;
    type ArchivedSetTable = ArchivedTable<SetOps<u32>, DefaultHasherSeed>;
    type ArchivedMapTable = ArchivedTable<MapOps<u32, u32>, DefaultHasherSeed>;

    fn set_of(keys: impl IntoIterator<Item = u32>) -> SetTable {
        let mut builder: Builder<SetOps<u32>, _, Portable> =
            Builder::with_hasher(DefaultHasherSeed::with_seed(1));
        for k in keys {
            builder.insert(k);
        }
        builder.build()
    }

    fn map_of(entries: impl IntoIterator<Item = (u32, u32)>) -> MapTable {
        let mut builder: Builder<MapOps<u32, u32>, _, Portable> =
            Builder::with_hasher(DefaultHasherSeed::with_seed(1));
        for (k, v) in entries {
            builder.map_insert(k, v);
        }
        builder.build()
    }

    #[test]
    fn an_archived_table_answers_the_same_queries_as_the_live_one() {
        let table = set_of(0..16);
        let bytes = rkyv::to_bytes::<Error>(&table).unwrap();
        let archived = rkyv::access::<ArchivedSetTable, Error>(&bytes).unwrap();

        assert_eq!(archived.len(), table.len());
        assert!(!archived.is_empty());
        assert_eq!(archived.hasher().seed(), 1);
        assert_eq!(archived.iter().count(), 16);
        assert_eq!(archived.as_slice().len(), 16);

        for k in 0..16u32 {
            let index = archived.get_index(&k).expect("archived key present");
            assert_eq!(index, table.get_index(&k).unwrap());
            assert!(archived.contains(&k));
            assert_eq!(archived.get(&k).unwrap().to_native(), k);
            assert_eq!(archived.index(index).unwrap().to_native(), k);

            // SAFETY: the archive is not empty and `index` came from `get_index`.
            unsafe {
                assert_eq!(archived.get_index_unchecked(&k), index);
                assert_eq!(archived.get_unchecked(&k).to_native(), k);
                assert_eq!(archived.index_unchecked(index).to_native(), k);
            }
        }

        assert_eq!(archived.get_index(&99u32), None);
        assert!(!archived.contains(&99u32));
        assert!(archived.get(&99u32).is_none());
        assert!(archived.index(16).is_none());
    }

    #[test]
    fn an_empty_archive_answers_without_touching_an_entry() {
        let table = set_of(core::iter::empty());
        let bytes = rkyv::to_bytes::<Error>(&table).unwrap();
        let archived = rkyv::access::<ArchivedSetTable, Error>(&bytes).unwrap();

        assert!(archived.is_empty());
        assert_eq!(archived.len(), 0);
        assert_eq!(archived.get_index(&1u32), None);
        assert!(!archived.contains(&1u32));
        assert!(archived.get(&1u32).is_none());
    }

    #[test]
    fn archived_map_accessors_split_entries_into_keys_and_values() {
        let table = map_of((0..8).map(|k| (k, k * 10)));
        let bytes = rkyv::to_bytes::<Error>(&table).unwrap();
        let archived = rkyv::access::<ArchivedMapTable, Error>(&bytes).unwrap();

        for k in 0..8u32 {
            let index = archived.get_index(&k).unwrap();
            let (key, value) = archived.map_index(index).unwrap();
            assert_eq!((key.to_native(), value.to_native()), (k, k * 10));

            let (key, value) = archived.map_get_key_value(&k).unwrap();
            assert_eq!((key.to_native(), value.to_native()), (k, k * 10));
            assert_eq!(archived.map_get(&k).unwrap().to_native(), k * 10);

            // SAFETY: the archive is not empty and `index` came from `get_index`.
            unsafe {
                let (key, value) = archived.map_index_unchecked(index);
                assert_eq!((key.to_native(), value.to_native()), (k, k * 10));
                let (key, value) = archived.map_get_key_value_unchecked(&k);
                assert_eq!((key.to_native(), value.to_native()), (k, k * 10));
                assert_eq!(archived.map_get_unchecked(&k).to_native(), k * 10);
            }
        }

        assert!(archived.map_index(99).is_none());
        assert!(archived.map_get_key_value(&99u32).is_none());
        assert!(archived.map_get(&99u32).is_none());
    }

    #[test]
    fn sealed_accessors_rewrite_values_in_place() {
        let table = set_of(0..10);
        let mut bytes = rkyv::to_bytes::<Error>(&table).unwrap();

        {
            let archived = rkyv::access_mut::<ArchivedSetTable, Error>(&mut bytes).unwrap();
            let value = ArchivedSetTable::get_seal(archived, &3u32).expect("present");
            *Seal::unseal(value) = 300.into();
        }
        {
            // Address a slot the previous edit did not touch, so the two are independent.
            let index = table.get_index(&9u32).expect("present");
            let archived = rkyv::access_mut::<ArchivedSetTable, Error>(&mut bytes).unwrap();
            let value = ArchivedSetTable::index_seal(archived, index).expect("in bounds");
            *Seal::unseal(value) = 900.into();
        }

        let archived = rkyv::access::<ArchivedSetTable, Error>(&bytes).unwrap();
        let seen = archived.iter().map(|e| e.to_native()).collect::<Vec<_>>();
        assert!(seen.contains(&300), "{seen:?}");
        assert!(seen.contains(&900), "{seen:?}");
        // Only the two targeted entries changed.
        assert_eq!(seen.len(), 10);
        for untouched in [0u32, 1, 2, 4, 5, 6, 7, 8] {
            assert!(seen.contains(&untouched), "{untouched} disturbed: {seen:?}");
        }

        // A seal cannot reach outside the table it came from.
        let mut bytes = rkyv::to_bytes::<Error>(&table).unwrap();
        let archived = rkyv::access_mut::<ArchivedSetTable, Error>(&mut bytes).unwrap();
        assert!(ArchivedSetTable::get_seal(archived, &99u32).is_none());
        let archived = rkyv::access_mut::<ArchivedSetTable, Error>(&mut bytes).unwrap();
        assert!(ArchivedSetTable::index_seal(archived, 10).is_none());
    }

    #[test]
    fn unchecked_sealed_accessors_rewrite_values_in_place() {
        let table = set_of(0..8);
        let mut bytes = rkyv::to_bytes::<Error>(&table).unwrap();

        {
            let archived = rkyv::access_mut::<ArchivedSetTable, Error>(&mut bytes).unwrap();
            // SAFETY: the archive is not empty.
            let value = unsafe { ArchivedSetTable::get_unchecked_seal(archived, &3u32) };
            *Seal::unseal(value) = 300.into();
        }
        {
            // Address the slot holding a key no other edit touches.
            let index = table.get_index(&7u32).expect("present");
            let archived = rkyv::access_mut::<ArchivedSetTable, Error>(&mut bytes).unwrap();
            // SAFETY: `index` came from `get_index`, so it is in bounds.
            let value = unsafe { ArchivedSetTable::index_unchecked_seal(archived, index) };
            *Seal::unseal(value) = 700.into();
        }
        {
            let archived = rkyv::access_mut::<ArchivedSetTable, Error>(&mut bytes).unwrap();
            // SAFETY: only values are written through the iterator, never keys.
            for entry in unsafe { ArchivedSetTable::iter_mut(archived) } {
                if entry.to_native() == 5 {
                    *entry = 500.into();
                }
            }
        }

        let archived = rkyv::access::<ArchivedSetTable, Error>(&bytes).unwrap();
        let seen = archived.iter().map(|e| e.to_native()).collect::<Vec<_>>();
        for expected in [300u32, 700, 500] {
            assert!(seen.contains(&expected), "{expected} missing from {seen:?}");
        }
        for untouched in [0u32, 1, 2, 4, 6] {
            assert!(seen.contains(&untouched), "{untouched} disturbed: {seen:?}");
        }
    }

    #[test]
    fn sealed_map_accessors_reach_values_and_leave_keys_shared() {
        let table = map_of((0..8).map(|k| (k, k * 10)));
        let mut bytes = rkyv::to_bytes::<Error>(&table).unwrap();

        {
            let archived = rkyv::access_mut::<ArchivedMapTable, Error>(&mut bytes).unwrap();
            let (key, value) = ArchivedMapTable::map_get_key_value_seal(archived, &1u32).unwrap();
            assert_eq!(key.to_native(), 1);
            *Seal::unseal(value) = 111.into();
        }
        {
            let archived = rkyv::access_mut::<ArchivedMapTable, Error>(&mut bytes).unwrap();
            let value = ArchivedMapTable::map_get_seal(archived, &2u32).unwrap();
            *Seal::unseal(value) = 222.into();
        }
        {
            let archived = rkyv::access_mut::<ArchivedMapTable, Error>(&mut bytes).unwrap();
            let index = archived.get_index(&3u32).unwrap();
            let (key, value) = ArchivedMapTable::map_index_seal(archived, index).unwrap();
            assert_eq!(key.to_native(), 3);
            *Seal::unseal(value) = 333.into();
        }
        {
            let archived = rkyv::access_mut::<ArchivedMapTable, Error>(&mut bytes).unwrap();
            // SAFETY: the archive is not empty.
            let (_, value) =
                unsafe { ArchivedMapTable::map_get_key_value_unchecked_seal(archived, &4u32) };
            *Seal::unseal(value) = 444.into();
        }
        {
            let archived = rkyv::access_mut::<ArchivedMapTable, Error>(&mut bytes).unwrap();
            // SAFETY: the archive is not empty.
            let value = unsafe { ArchivedMapTable::map_get_unchecked_seal(archived, &5u32) };
            *Seal::unseal(value) = 555.into();
        }
        {
            let archived = rkyv::access_mut::<ArchivedMapTable, Error>(&mut bytes).unwrap();
            let index = archived.get_index(&6u32).unwrap();
            // SAFETY: `index` came from `get_index`, so it is in bounds.
            let (_, value) = unsafe { ArchivedMapTable::map_index_unchecked_seal(archived, index) };
            *Seal::unseal(value) = 666.into();
        }

        let archived = rkyv::access::<ArchivedMapTable, Error>(&bytes).unwrap();
        for (k, expected) in [
            (1u32, 111u32),
            (2, 222),
            (3, 333),
            (4, 444),
            (5, 555),
            (6, 666),
        ] {
            assert_eq!(archived.map_get(&k).unwrap().to_native(), expected);
        }
        // The untouched entry still holds its original value, so nothing else was disturbed.
        assert_eq!(archived.map_get(&7u32).unwrap().to_native(), 70);
    }

    #[test]
    fn disjoint_seals_hand_out_independent_values() {
        let table = map_of((0..8).map(|k| (k, k * 10)));
        let mut bytes = rkyv::to_bytes::<Error>(&table).unwrap();

        {
            let archived = rkyv::access_mut::<ArchivedMapTable, Error>(&mut bytes).unwrap();
            let [a, b, missing] =
                ArchivedMapTable::map_get_disjoint_seal(archived, [&1u32, &2u32, &99u32]);
            *Seal::unseal(a.unwrap()) = 111.into();
            *Seal::unseal(b.unwrap()) = 222.into();
            assert!(missing.is_none());
        }
        {
            let archived = rkyv::access_mut::<ArchivedMapTable, Error>(&mut bytes).unwrap();
            let [a, missing] =
                ArchivedMapTable::map_get_disjoint_key_value_seal(archived, [&3u32, &99u32]);
            let (key, value) = a.unwrap();
            assert_eq!(key.to_native(), 3);
            *Seal::unseal(value) = 333.into();
            assert!(missing.is_none());
        }
        {
            let archived = rkyv::access_mut::<ArchivedMapTable, Error>(&mut bytes).unwrap();
            // SAFETY: the keys are pairwise distinct, so they cannot share an entry.
            let [a, b] = unsafe {
                ArchivedMapTable::map_get_disjoint_unchecked_seal(archived, [&4u32, &5u32])
            };
            *Seal::unseal(a.unwrap()) = 444.into();
            *Seal::unseal(b.unwrap()) = 555.into();
        }
        {
            let archived = rkyv::access_mut::<ArchivedMapTable, Error>(&mut bytes).unwrap();
            // SAFETY: the keys are pairwise distinct, so they cannot share an entry.
            let [a, b] = unsafe {
                ArchivedMapTable::map_get_disjoint_key_value_unchecked_seal(
                    archived,
                    [&6u32, &7u32],
                )
            };
            *Seal::unseal(a.unwrap().1) = 666.into();
            *Seal::unseal(b.unwrap().1) = 777.into();
        }

        let archived = rkyv::access::<ArchivedMapTable, Error>(&bytes).unwrap();
        for k in 1..8u32 {
            assert_eq!(archived.map_get(&k).unwrap().to_native(), k * 111);
        }
    }

    #[test]
    fn the_checked_disjoint_seals_refuse_to_alias() {
        let table = map_of((0..8).map(|k| (k, k * 10)));
        let mut bytes = rkyv::to_bytes::<Error>(&table).unwrap();

        let aliased = catch_unwind(AssertUnwindSafe(|| {
            let archived = rkyv::access_mut::<ArchivedMapTable, Error>(&mut bytes).unwrap();
            ArchivedMapTable::map_get_disjoint_seal(archived, [&1u32, &1u32]);
        }));
        assert!(
            aliased.is_err(),
            "two seals over one entry must not be handed out"
        );

        let aliased = catch_unwind(AssertUnwindSafe(|| {
            let archived = rkyv::access_mut::<ArchivedMapTable, Error>(&mut bytes).unwrap();
            ArchivedMapTable::map_get_disjoint_key_value_seal(archived, [&2u32, &2u32]);
        }));
        assert!(
            aliased.is_err(),
            "two seals over one entry must not be handed out"
        );
    }

    #[test]
    fn validation_rejects_an_index_that_points_outside_the_entries() {
        // The unchecked accessors trust `indices` to be in range, so a tampered archive has to
        // be refused rather than accepted and dereferenced.
        let mut builder: Builder<OrderedSetOps<u32>, _, Portable> =
            Builder::with_hasher(DefaultHasherSeed::with_seed(1));
        for k in 0..8u32 {
            builder.insert(k);
        }
        let table = builder.build();

        let mut bytes = rkyv::to_bytes::<Error>(&table).unwrap();
        type Archived8 = ArchivedTable<OrderedSetOps<u32>, DefaultHasherSeed>;
        assert!(rkyv::access::<Archived8, Error>(&bytes).is_ok());

        let offset = {
            let archived = rkyv::access::<Archived8, Error>(&bytes).unwrap();
            archived.indices.as_ptr() as usize - bytes.as_ptr() as usize
        };
        // Any byte pattern here yields an index of at least 255, well past the eight entries.
        bytes.as_mut_slice()[offset] = 0xFF;

        assert!(
            rkyv::access::<Archived8, Error>(&bytes).is_err(),
            "an out-of-range slot must fail validation",
        );
    }

    #[test]
    fn an_archived_table_shows_the_parameters_that_address_its_entries() {
        let table = map_of([(1u32, 10u32)]);
        let bytes = rkyv::to_bytes::<Error>(&table).unwrap();
        let archived = rkyv::access::<ArchivedMapTable, Error>(&bytes).unwrap();

        let shown = alloc::format!("{archived:?}");
        for field in [
            "global_param",
            "params",
            "indices",
            "entries",
            "hasher_builder",
        ] {
            assert!(shown.contains(field), "`{field}` missing from {shown}");
        }
        assert!(shown.contains("10"), "{shown}");
    }

    #[test]
    fn an_archive_deserializes_back_into_a_live_table() {
        let table = map_of((0..8).map(|k| (k, k * 10)));
        let bytes = rkyv::to_bytes::<Error>(&table).unwrap();
        let archived = rkyv::access::<ArchivedMapTable, Error>(&bytes).unwrap();

        let round_tripped: MapTable = rkyv::deserialize::<_, Error>(archived).unwrap();

        assert_eq!(round_tripped.len(), 8);
        assert_eq!(round_tripped.hasher().seed(), 1);
        for k in 0..8u32 {
            assert_eq!(round_tripped.map_get(&k), Some(&(k * 10)));
        }
    }

    #[test]
    fn archived_equality_ignores_the_entry_order() {
        let a = map_of((0..8).map(|k| (k, k * 10)));
        let b = {
            let mut builder: Builder<MapOps<u32, u32>, _, Portable> =
                Builder::with_hasher(DefaultHasherSeed::with_seed(9));
            for k in (0..8u32).rev() {
                builder.map_insert(k, k * 10);
            }
            builder.build()
        };
        let shorter = map_of((0..7).map(|k| (k, k * 10)));
        let differing = map_of((0..8).map(|k| (k, if k == 3 { 99 } else { k * 10 })));

        let (ba, bb) = (
            rkyv::to_bytes::<Error>(&a).unwrap(),
            rkyv::to_bytes::<Error>(&b).unwrap(),
        );
        let (bs, bd) = (
            rkyv::to_bytes::<Error>(&shorter).unwrap(),
            rkyv::to_bytes::<Error>(&differing).unwrap(),
        );

        let aa = rkyv::access::<ArchivedMapTable, Error>(&ba).unwrap();
        let ab = rkyv::access::<ArchivedMapTable, Error>(&bb).unwrap();
        let as_ = rkyv::access::<ArchivedMapTable, Error>(&bs).unwrap();
        let ad = rkyv::access::<ArchivedMapTable, Error>(&bd).unwrap();

        assert_eq!(aa, ab);
        assert_ne!(aa, as_);
        assert_ne!(aa, ad);
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use super::*;
    use crate::portability::{MapOps, SetOps};
    use crate::table::Builder;

    use alloc::string::String;
    use alloc::vec::Vec;
    use portable::{PortableEq, PortableHash};
    use rkyv::rancor::Error;

    // Reaching these methods means naming an archived type that implements `serde::Serialize`.
    // rkyv's own archived types never will — `ArchivedString` and `u32_le` are foreign, so
    // nobody downstream can implement it for them — but a newtype makes the archived form local
    // and the impl writable. Both halves of an entry need it, integers included.
    macro_rules! newtype {
        ($name:ident, $archived:ident, $inner:ty, $ser:ident, $native:ident) => {
            #[derive(rkyv::Archive, rkyv::Serialize, serde::Serialize, Debug, PartialEq, Eq)]
            #[serde(transparent)]
            struct $name($inner);

            impl PortableHash for $name {
                fn portable_hash<H: core::hash::Hasher>(&self, state: &mut H) {
                    self.0.portable_hash(state);
                }
            }

            impl PortableEq<$name> for $name {
                fn portable_eq(&self, other: &$name) -> bool {
                    self.0.portable_eq(&other.0)
                }
            }

            impl serde::Serialize for $archived {
                fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                    s.$ser(self.0.$native())
                }
            }
        };
    }

    newtype!(Key, ArchivedKey, String, serialize_str, as_str);
    newtype!(Count, ArchivedCount, u32, serialize_u32, to_native);

    fn to_json(value: impl serde::Serialize) -> String {
        serde_json::to_string(&value).unwrap()
    }

    #[test]
    fn an_archived_map_serializes_exactly_like_the_map_it_came_from() {
        let mut builder: Builder<MapOps<Key, Count>, _, Portable> =
            Builder::with_hasher(DefaultHasherSeed::with_seed(7));
        builder.insert((Key("a".into()), Count(1)));
        builder.insert((Key("bb".into()), Count(2)));
        let table = builder.build();

        let bytes = rkyv::to_bytes::<Error>(&table).unwrap();
        let archived =
            rkyv::access::<ArchivedTable<MapOps<Key, Count>, DefaultHasherSeed>, Error>(&bytes)
                .unwrap();

        let mut out = Vec::new();
        archived
            .serialize_map(&mut serde_json::Serializer::new(&mut out))
            .unwrap();

        // The two forms have to be interchangeable to a `serde` consumer, so an archive can be
        // re-serialized without deserializing it first.
        assert_eq!(String::from_utf8(out).unwrap(), to_json(&table));
    }

    #[test]
    fn an_archived_set_serializes_exactly_like_the_set_it_came_from() {
        let mut builder: Builder<SetOps<Key>, _, Portable> =
            Builder::with_hasher(DefaultHasherSeed::with_seed(7));
        builder.insert(Key("a".into()));
        builder.insert(Key("bb".into()));
        let table = builder.build();

        let bytes = rkyv::to_bytes::<Error>(&table).unwrap();
        let archived =
            rkyv::access::<ArchivedTable<SetOps<Key>, DefaultHasherSeed>, Error>(&bytes).unwrap();

        let mut out = Vec::new();
        archived
            .serialize_set(&mut serde_json::Serializer::new(&mut out))
            .unwrap();

        assert_eq!(String::from_utf8(out).unwrap(), to_json(&table));
    }
}
