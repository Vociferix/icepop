//! The portability type parameter and the trait indirection it selects.
//!
//! Every collection carries a `P` parameter that is either [`NonPortable`] or [`Portable`].
//! The lookup and build code is written once against the `*Ops` traits in this module, each of
//! which has one blanket implementation per marker: [`NonPortable`] resolves to the standard
//! [`Hash`]/[`BuildHasher`]/`Equivalent` interface, [`Portable`] to the equivalents from the
//! `portable` crate. Because both markers are uninhabited, `P` only ever selects
//! implementations; it never occupies space or reaches runtime.
//!
//! [`TableOps`] is a second, unrelated axis: it names the entry, key, value and index-array
//! types that distinguish a map from a set and an ordered collection from an unordered one.

#[cfg(feature = "serde")]
use crate::table::Table;

#[cfg(feature = "rkyv")]
use crate::table::rkyv::ArchivedTable;

use equivalent::Equivalent;
use portable::{PortableBuildHasher, PortableEq, PortableHash, PortableHasher};

use core::hash::{BuildHasher, Hash, Hasher};
use core::marker::PhantomData;

use alloc::boxed::Box;

/// Marker selecting the standard, process-local lookup interface.
///
/// Keys are hashed with [`Hash`] and compared with [`Equivalent`], and the hasher need only
/// implement [`BuildHasher`]. Nothing is required to be reproducible, so the hasher can be
/// chosen purely for speed, but the resulting collection is meaningful only inside the process
/// that built it and supports neither `serde` nor `rkyv`. Use [`Portable`] for anything that
/// leaves the process.
///
/// This is the default for every collection, and the type is uninhabited: it exists only to
/// pick trait implementations.
///
/// # Example
///
/// ```
/// use icepop_phf::{DefaultHasherSeed, Map, NonPortable};
///
/// // The default `P`, so both annotations name the same type.
/// let implied: Map<&str, u32> = [("a", 1)].into_iter().collect();
/// let spelled: Map<&str, u32, DefaultHasherSeed, NonPortable> = [("a", 1)].into_iter().collect();
///
/// assert_eq!(implied.get("a"), spelled.get("a"));
/// ```
///
/// [`Equivalent`]: https://docs.rs/equivalent/1/equivalent/trait.Equivalent.html
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NonPortable {}

/// Marker selecting the platform-independent lookup interface.
///
/// Keys are hashed with [`PortableHash`] and compared with [`PortableEq`], and the hasher must
/// implement [`PortableBuildHasher`]. Those traits produce the same bytes and the same hash on
/// every target, so a collection built here can be serialized, moved to a machine with
/// different endianness or pointer width, and read there. Only these collections implement the
/// `serde` and `rkyv` traits.
///
/// The type is uninhabited: it exists only to pick trait implementations. The `Portable*`
/// aliases, such as [`PortableMap`](crate::PortableMap), name the collections that use it.
///
/// # Example
///
/// ```
/// use icepop_phf::{DefaultHasherSeed, Map, Portable, PortableMap};
///
/// // The alias and the spelled-out form are the same type.
/// let aliased: PortableMap<&str, u32> = [("a", 1)].into_iter().collect();
/// let spelled: Map<&str, u32, DefaultHasherSeed, Portable> = [("a", 1)].into_iter().collect();
///
/// assert_eq!(aliased.get("a"), spelled.get("a"));
/// ```
///
/// [`PortableBuildHasher`]: portable::PortableBuildHasher
/// [`PortableEq`]: portable::PortableEq
/// [`PortableHash`]: portable::PortableHash
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Portable {}

/// Builds hashers, under whichever hashing interface `P` selects.
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

/// Finishes a hash, under whichever hashing interface `P` selects.
pub trait HasherOps<P> {
    fn finish(&self) -> u64;
}

/// Feeds a value to a hasher, under whichever hashing interface `P` selects.
pub trait HashOps<S: BuildHasherOps<P>, P> {
    fn hash(&self, state: &mut S::Hasher);
}

/// Compares a lookup key against a stored key, under whichever interface `P` selects.
///
/// Both interfaces admit borrowed lookup keys: `Equivalent` for [`NonPortable`], and
/// `PortableEq`'s cross-type comparison for [`Portable`].
pub trait EqOps<K: ?Sized, P> {
    fn eq(&self, other: &K) -> bool;
}

/// The shape of a table: what an entry is, and whether hash slots are indirected.
///
/// One implementation per collection kind. A map entry is a `(K, V)` pair and a set entry is
/// the element itself, which is why `Key` and `Value` are named separately from `Entry`. An
/// ordered collection sets `Indices` to `Box<[u32]>` and [`HAVE_INDICES`](Self::HAVE_INDICES),
/// so a hash slot is looked up in that array to find the entry; an unordered one sets
/// `Indices` to `()` and uses the slot as the entry index directly.
pub trait TableOps: Sized {
    /// What one element of the entry array is: `(K, V)` for a map, `T` for a set.
    type Entry: Sized;

    /// The part of an entry that is hashed and compared.
    type Key: Sized;

    /// The part of an entry a map lookup returns; `()` for a set.
    type Value: Sized;

    /// `Box<[u32]>` mapping hash slots to entry indices, or `()` when slots are indices.
    type Indices: Sized;

    /// Whether [`Indices`](Self::Indices) is a real array, and so must be built and validated.
    const HAVE_INDICES: bool;

    /// Borrows the key out of an entry.
    fn get_key(entry: &Self::Entry) -> &Self::Key;

    /// Resolves a hash slot to an entry index.
    ///
    /// # Safety
    ///
    /// `slot` must be less than the number of entries.
    unsafe fn get_index(indices: &Self::Indices, slot: usize) -> usize;

    /// Checks that a table's arrays agree in length and that every index is in bounds.
    ///
    /// Deserialization builds a table from untrusted input, so the invariants the builder
    /// establishes must be re-established here before any unchecked access is allowed.
    #[cfg(feature = "serde")]
    fn verify<S>(table: &Table<Self, S, Portable>) -> bool;

    /// Borrows the key out of an archived entry.
    #[cfg(feature = "rkyv")]
    fn get_key_archived<'a>(
        entry: &'a rkyv::Archived<Self::Entry>,
    ) -> &'a rkyv::Archived<Self::Key>
    where
        Self::Entry: rkyv::Archive,
        Self::Key: rkyv::Archive,
        Self::Value: rkyv::Archive + 'a;

    /// Resolves a hash slot to an entry index, in an archived table.
    ///
    /// # Safety
    ///
    /// `slot` must be less than the number of entries.
    #[cfg(feature = "rkyv")]
    unsafe fn get_index_archived(indices: &rkyv::Archived<Self::Indices>, slot: usize) -> usize
    where
        Self::Indices: rkyv::Archive;

    /// [`verify`](Self::verify) for an archived table, run by `bytecheck` during validation.
    #[cfg(feature = "rkyv")]
    fn verify_archived<S>(table: &ArchivedTable<Self, S>) -> bool
    where
        S: rkyv::Archive,
        Self::Entry: rkyv::Archive,
        Self::Indices: rkyv::Archive;

    /// The archived index array, or an empty slice when there is none to serialize.
    #[cfg(all(feature = "rkyv", feature = "serde"))]
    fn archived_indices<S>(table: &ArchivedTable<Self, S>) -> &[rkyv::Archived<u32>]
    where
        S: rkyv::Archive,
        Self::Entry: rkyv::Archive,
        Self::Indices: rkyv::Archive;
}

/// [`TableOps`] for a map that keeps insertion order.
pub struct OrderedMapOps<K, V>(PhantomData<fn(K, V)>);

/// [`TableOps`] for a map whose entries are permuted into hash-slot order.
pub struct MapOps<K, V>(PhantomData<fn(K, V)>);

/// [`TableOps`] for a set that keeps insertion order.
pub struct OrderedSetOps<T>(PhantomData<fn(T)>);

/// [`TableOps`] for a set whose entries are permuted into hash-slot order.
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
        // SAFETY: the caller guarantees `slot` is less than the entry count, and an ordered table's
        // index array is exactly that long.
        (unsafe { *indices.get_unchecked(slot) }) as usize
    }

    #[cfg(feature = "serde")]
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
        // SAFETY: `Self::Entry` is `(K, V)`, and rkyv archives a pair as `ArchivedTuple2`, so
        // source and target are the same type; the associated type hides that from the compiler.
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
        // SAFETY: the caller guarantees `slot` is less than the entry count, and an ordered table's
        // index array is exactly that long.
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

    #[cfg(feature = "serde")]
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
        // SAFETY: `Self::Entry` is `(K, V)`, and rkyv archives a pair as `ArchivedTuple2`, so
        // source and target are the same type; the associated type hides that from the compiler.
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
        // SAFETY: the caller guarantees `slot` is less than the entry count, and an ordered table's
        // index array is exactly that long.
        (unsafe { *indices.get_unchecked(slot) }) as usize
    }

    #[cfg(feature = "serde")]
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
        // SAFETY: the caller guarantees `slot` is less than the entry count, and an ordered table's
        // index array is exactly that long.
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

    #[cfg(feature = "serde")]
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
