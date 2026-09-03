//! The table every collection in this crate wraps.
//!
//! One implementation serves maps and sets, ordered and unordered, portable and not: `O` supplies
//! the entry shape via [`TableOps`](crate::portability::TableOps) and `P` supplies the hashing and
//! comparison interface via the `*Ops` traits. The public collections are thin newtypes that
//! re-expose the slice of this API that makes sense for them, under concrete trait bounds.
//!
//! # Lookup
//!
//! The minimal perfect hash function is the two-round CHD scheme the builder constructs:
//!
//! 1. Hash `global_param` then the key, and reduce modulo `len` to get a bucket.
//! 2. Feed that bucket's displacement `params[bucket]` into the *same* hasher and reduce again.
//!    The result is a slot, unique across all keys by construction.
//! 3. Map the slot to an entry index. Ordered tables read `indices[slot]`; unordered tables were
//!    permuted at build time so that the slot is already the index.
//!
//! Nothing about this rules out a key that was never inserted: an absent key still lands on some
//! slot, and some entry. That is why every lookup that promises an answer compares the key against
//! the entry it found, and why the `*_unchecked` variants are `unsafe` only in that they skip the
//! empty-table check.

use crate::portability::{BuildHasherOps, EqOps, HashOps, HasherOps, NonPortable, TableOps};

use portable::DefaultHasherSeed;

use core::marker::PhantomData;

use alloc::boxed::Box;

#[cfg(feature = "rkyv")]
pub mod rkyv;

mod builder;

pub use builder::Builder;

/// A built table: entries plus the parameters that address them.
///
/// # Invariants
///
/// `params` and `entries` have the same length, `indices` too when
/// [`HAVE_INDICES`](TableOps::HAVE_INDICES), and every value in `indices` is a valid index into
/// `entries`. The builder establishes these; [`TableOps::verify`](crate::portability::TableOps::verify)
/// re-establishes them for a table that arrived by deserialization.
pub struct Table<O: TableOps, S = DefaultHasherSeed, P = NonPortable> {
    /// Mixed into the hasher before every key. Bumped when a seed fails to yield a perfect hash.
    pub(crate) global_param: u8,
    /// Per-bucket displacement, indexed by the first hash reduced modulo `entries.len()`.
    pub(crate) params: Box<[u32]>,
    /// Slot to entry index, or `()` when entries were permuted so that slot equals index.
    pub(crate) indices: O::Indices,
    pub(crate) entries: Box<[O::Entry]>,
    pub(crate) hasher_builder: S,
    pub(crate) _portable: PhantomData<P>,
}

impl<O, S, P> Table<O, S, P>
where
    O: TableOps,
{
    pub fn hasher(&self) -> &S {
        &self.hasher_builder
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn iter(&self) -> core::slice::Iter<'_, O::Entry> {
        self.entries.iter()
    }

    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, O::Entry> {
        self.entries.iter_mut()
    }

    pub fn as_slice(&self) -> &[O::Entry] {
        &self.entries
    }

    pub fn index(&self, index: usize) -> Option<&O::Entry> {
        self.entries.get(index)
    }

    pub fn index_mut(&mut self, index: usize) -> Option<&mut O::Entry> {
        self.entries.get_mut(index)
    }

    /// # Safety
    ///
    /// `index` must be less than [`len`](Self::len).
    pub unsafe fn index_unchecked(&self, index: usize) -> &O::Entry {
        // SAFETY: the caller guarantees `index` is within the entry array.
        unsafe { self.entries.get_unchecked(index) }
    }

    /// # Safety
    ///
    /// `index` must be less than [`len`](Self::len).
    pub unsafe fn index_unchecked_mut(&mut self, index: usize) -> &mut O::Entry {
        // SAFETY: the caller guarantees `index` is within the entry array.
        unsafe { self.entries.get_unchecked_mut(index) }
    }
}

impl<K, V, O, S, P> Table<O, S, P>
where
    O: TableOps<Entry = (K, V), Key = K, Value = V>,
{
    pub fn map_index(&self, index: usize) -> Option<(&K, &V)> {
        self.index(index).map(|(k, v)| (k, v))
    }

    pub fn map_index_mut(&mut self, index: usize) -> Option<(&K, &mut V)> {
        self.index_mut(index).map(|(k, v)| (&*k, v))
    }

    /// # Safety
    ///
    /// `index` must be less than [`len`](Self::len).
    pub unsafe fn map_index_unchecked(&self, index: usize) -> (&K, &V) {
        // SAFETY: `index_unchecked` has this function's contract, which the caller upheld.
        let (k, v) = unsafe { self.index_unchecked(index) };
        (k, v)
    }

    /// # Safety
    ///
    /// `index` must be less than [`len`](Self::len).
    pub unsafe fn map_index_unchecked_mut(&mut self, index: usize) -> (&K, &mut V) {
        // SAFETY: `index_unchecked_mut` has this function's contract, which the caller upheld.
        let (k, v) = unsafe { self.index_unchecked_mut(index) };
        (&*k, v)
    }
}

impl<O, S, P> Table<O, S, P>
where
    O: TableOps,
    S: BuildHasherOps<P>,
    u8: HashOps<S, P>,
    u32: HashOps<S, P>,
{
    /// Runs the two hash rounds and returns the entry index the slot resolves to.
    ///
    /// The result is in bounds for any key, present or not, so callers that need an answer about
    /// `key` must still compare it against the entry found.
    ///
    /// # Safety
    ///
    /// The table must not be empty. The modulus is the entry count, and the hint that it is
    /// non-zero is what lets the two reductions compile without a zero check.
    pub unsafe fn get_index_unchecked<Q>(&self, key: &Q) -> usize
    where
        Q: HashOps<S, P> + EqOps<O::Key, P> + ?Sized,
    {
        let modulus = self.entries.len() as u64;
        // SAFETY: the caller guarantees the table is not empty. Stating that here is what lets the
        // two reductions below compile without a division-by-zero check.
        unsafe {
            core::hint::assert_unchecked(modulus != 0);
        }

        let mut hasher = self.hasher_builder.build_hasher();

        HashOps::<S, P>::hash(&self.global_param, &mut hasher);
        key.hash(&mut hasher);
        let param_idx = (hasher.finish() % modulus) as usize;
        // SAFETY: `param_idx` is a remainder modulo the entry count, and `params` holds exactly
        // that many elements.
        let param = unsafe { *self.params.get_unchecked(param_idx) };

        HashOps::<S, P>::hash(&param, &mut hasher);
        let index_idx = (hasher.finish() % modulus) as usize;
        // SAFETY: `index_idx` is a remainder modulo the entry count, which is the bound `get_index`
        // requires of a slot.
        unsafe { O::get_index(&self.indices, index_idx) }
    }

    pub fn get_index<Q>(&self, key: &Q) -> Option<usize>
    where
        Q: HashOps<S, P> + EqOps<O::Key, P> + ?Sized,
    {
        if self.entries.is_empty() {
            return None;
        }

        // SAFETY: the early return above establishes that the table is not empty.
        let index = unsafe { self.get_index_unchecked(key) };
        // SAFETY: `get_index_unchecked` resolves to an index within the entry array.
        let entry = unsafe { self.entries.get_unchecked(index) };
        key.eq(O::get_key(entry)).then_some(index)
    }

    pub fn contains<Q>(&self, key: &Q) -> bool
    where
        Q: HashOps<S, P> + EqOps<O::Key, P> + ?Sized,
    {
        if self.entries.is_empty() {
            return false;
        }

        // SAFETY: the early return above establishes that the table is not empty.
        let index = unsafe { self.get_index_unchecked(key) };
        // SAFETY: `get_index_unchecked` resolves to an index within the entry array.
        let entry = unsafe { self.entries.get_unchecked(index) };
        key.eq(O::get_key(entry))
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&O::Entry>
    where
        Q: HashOps<S, P> + EqOps<O::Key, P> + ?Sized,
    {
        if self.entries.is_empty() {
            return None;
        }

        // SAFETY: the early return above establishes that the table is not empty.
        let index = unsafe { self.get_index_unchecked(key) };
        // SAFETY: `get_index_unchecked` resolves to an index within the entry array.
        let entry = unsafe { self.entries.get_unchecked(index) };
        key.eq(O::get_key(entry)).then_some(entry)
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut O::Entry>
    where
        Q: HashOps<S, P> + EqOps<O::Key, P> + ?Sized,
    {
        if self.entries.is_empty() {
            return None;
        }

        // SAFETY: the early return above establishes that the table is not empty.
        let index = unsafe { self.get_index_unchecked(key) };
        // SAFETY: `get_index_unchecked` resolves to an index within the entry array.
        let entry = unsafe { self.entries.get_unchecked_mut(index) };
        key.eq(O::get_key(entry)).then_some(entry)
    }

    /// # Safety
    ///
    /// The table must not be empty. An absent key yields an arbitrary entry.
    pub unsafe fn get_unchecked<Q>(&self, key: &Q) -> &O::Entry
    where
        Q: HashOps<S, P> + EqOps<O::Key, P> + ?Sized,
    {
        // SAFETY: `get_index_unchecked` has this function's contract, which the caller upheld.
        let idx = unsafe { self.get_index_unchecked(key) };
        // SAFETY: `get_index_unchecked` resolves to an index within the entry array.
        unsafe { self.entries.get_unchecked(idx) }
    }

    /// # Safety
    ///
    /// The table must not be empty. An absent key yields an arbitrary entry.
    pub unsafe fn get_unchecked_mut<Q>(&mut self, key: &Q) -> &mut O::Entry
    where
        Q: HashOps<S, P> + EqOps<O::Key, P> + ?Sized,
    {
        // SAFETY: `get_index_unchecked` has this function's contract, which the caller upheld.
        let idx = unsafe { self.get_index_unchecked(key) };
        // SAFETY: `get_index_unchecked` resolves to an index within the entry array.
        unsafe { self.entries.get_unchecked_mut(idx) }
    }
}

impl<K, V, O, S, P> Table<O, S, P>
where
    O: TableOps<Entry = (K, V), Key = K, Value = V>,
    S: BuildHasherOps<P>,
    u8: HashOps<S, P>,
    u32: HashOps<S, P>,
{
    pub fn map_get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        Q: HashOps<S, P> + EqOps<K, P> + ?Sized,
    {
        self.get(key).map(|(k, v)| (k, v))
    }

    pub fn map_get_key_value_mut<Q>(&mut self, key: &Q) -> Option<(&K, &mut V)>
    where
        Q: HashOps<S, P> + EqOps<K, P> + ?Sized,
    {
        self.get_mut(key).map(|(k, v)| (&*k, v))
    }

    pub fn map_get<'a, Q>(&'a self, key: &Q) -> Option<&'a V>
    where
        Q: HashOps<S, P> + EqOps<K, P> + ?Sized,
        K: 'a,
    {
        self.get(key).map(|(_, v)| v)
    }

    pub fn map_get_mut<'a, Q>(&'a mut self, key: &Q) -> Option<&'a mut V>
    where
        Q: HashOps<S, P> + EqOps<K, P> + ?Sized,
        K: 'a,
    {
        self.get_mut(key).map(|(_, v)| v)
    }

    /// # Safety
    ///
    /// The table must not be empty. An absent key yields an arbitrary entry.
    pub unsafe fn map_get_key_value_unchecked<Q>(&self, key: &Q) -> (&K, &V)
    where
        Q: HashOps<S, P> + EqOps<K, P> + ?Sized,
    {
        // SAFETY: `get_unchecked` has this function's contract, which the caller upheld.
        let (k, v) = unsafe { self.get_unchecked(key) };
        (k, v)
    }

    /// # Safety
    ///
    /// The table must not be empty. An absent key yields an arbitrary entry.
    pub unsafe fn map_get_key_value_unchecked_mut<Q>(&mut self, key: &Q) -> (&K, &mut V)
    where
        Q: HashOps<S, P> + EqOps<K, P> + ?Sized,
    {
        // SAFETY: `get_unchecked_mut` has this function's contract, which the caller upheld.
        let (k, v) = unsafe { self.get_unchecked_mut(key) };
        (&*k, v)
    }

    /// # Safety
    ///
    /// The table must not be empty. An absent key yields an arbitrary value.
    pub unsafe fn map_get_unchecked<'a, Q>(&'a self, key: &Q) -> &'a V
    where
        Q: HashOps<S, P> + EqOps<K, P> + ?Sized,
        K: 'a,
    {
        // SAFETY: `get_unchecked` has this function's contract, which the caller upheld.
        &unsafe { self.get_unchecked(key) }.1
    }

    /// # Safety
    ///
    /// The table must not be empty. An absent key yields an arbitrary value.
    pub unsafe fn map_get_unchecked_mut<'a, Q>(&'a mut self, key: &Q) -> &'a mut V
    where
        Q: HashOps<S, P> + EqOps<K, P> + ?Sized,
        K: 'a,
    {
        // SAFETY: `get_unchecked_mut` has this function's contract, which the caller upheld.
        &mut unsafe { self.get_unchecked_mut(key) }.1
    }

    /// # Panics
    ///
    /// Panics if two keys resolve to the same entry.
    pub fn map_get_disjoint_key_value_mut<'a, Q, const N: usize>(
        &'a mut self,
        keys: [&Q; N],
    ) -> [Option<(&'a K, &'a mut V)>; N]
    where
        Q: HashOps<S, P> + EqOps<K, P> + ?Sized,
        K: 'a,
    {
        let indices = keys.map(|key| self.get_index(key));
        assert!(unique_indices(&indices), "duplicate key found");
        // Taken once, before any entry is borrowed: reborrowing `self.entries` again would
        // cover the whole array and invalidate the references already handed out.
        let entries = self.entries.as_mut_ptr();
        indices.map(|idx| {
            idx.map(|idx| {
                // SAFETY: `idx` came from `get_index`, so it is within the entry array, and the
                // assert above rules out two keys sharing one, so no two borrows alias.
                let (k, v) = unsafe { &mut *entries.add(idx) };
                (&*k, v)
            })
        })
    }

    /// # Panics
    ///
    /// Panics if two keys resolve to the same entry.
    pub fn map_get_disjoint_mut<'a, Q, const N: usize>(
        &'a mut self,
        keys: [&Q; N],
    ) -> [Option<&'a mut V>; N]
    where
        Q: HashOps<S, P> + EqOps<K, P> + ?Sized,
        K: 'a,
    {
        let indices = keys.map(|key| self.get_index(key));
        assert!(unique_indices(&indices), "duplicate key found");
        // Taken once, before any entry is borrowed: reborrowing `self.entries` again would
        // cover the whole array and invalidate the references already handed out.
        let entries = self.entries.as_mut_ptr();
        // SAFETY: each `idx` came from `get_index`, so it is within the entry array, and the assert
        // above rules out two keys sharing one, so no two borrows alias.
        indices.map(|idx| idx.map(|idx| unsafe { &mut (*entries.add(idx)).1 }))
    }

    /// # Safety
    ///
    /// No two keys may resolve to the same entry; each is handed out as a distinct `&mut`.
    pub unsafe fn map_get_disjoint_key_value_unchecked_mut<'a, Q, const N: usize>(
        &'a mut self,
        keys: [&Q; N],
    ) -> [Option<(&'a K, &'a mut V)>; N]
    where
        Q: HashOps<S, P> + EqOps<K, P> + ?Sized,
        K: 'a,
    {
        let indices = keys.map(|key| self.get_index(key));
        // Taken once, after the last shared borrow and before any entry is borrowed: either a
        // further `get_index` or a second `as_mut_ptr` would reborrow the whole array and
        // invalidate the references already handed out.
        let entries = self.entries.as_mut_ptr();
        indices.map(|idx| {
            idx.map(|idx| {
                // SAFETY: `idx` came from `get_index`, so it is within the entry array, and the
                // caller guarantees no two keys share an entry, so no two borrows alias.
                let (k, v) = unsafe { &mut *entries.add(idx) };
                (&*k, v)
            })
        })
    }

    /// # Safety
    ///
    /// No two keys may resolve to the same entry; each is handed out as a distinct `&mut`.
    pub unsafe fn map_get_disjoint_unchecked_mut<'a, Q, const N: usize>(
        &'a mut self,
        keys: [&Q; N],
    ) -> [Option<&'a mut V>; N]
    where
        Q: HashOps<S, P> + EqOps<K, P> + ?Sized,
        K: 'a,
    {
        let indices = keys.map(|key| self.get_index(key));
        // Taken once, after the last shared borrow and before any entry is borrowed: either a
        // further `get_index` or a second `as_mut_ptr` would reborrow the whole array and
        // invalidate the references already handed out.
        let entries = self.entries.as_mut_ptr();
        // SAFETY: each `idx` came from `get_index`, so it is within the entry array, and the caller
        // guarantees no two keys share an entry, so no two borrows alias.
        indices.map(|idx| idx.map(|idx| unsafe { &mut (*entries.add(idx)).1 }))
    }
}

// Hand-written rather than derived: a derive would demand `O: Debug` and `P: Debug` of the
// marker types, which carry no data and implement nothing.
impl<O, S, P> core::fmt::Debug for Table<O, S, P>
where
    O: TableOps,
    O::Entry: core::fmt::Debug,
    O::Indices: core::fmt::Debug,
    S: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Table")
            .field("global_param", &self.global_param)
            .field("params", &self.params)
            .field("indices", &self.indices)
            .field("entries", &self.entries)
            .field("hasher_builder", &self.hasher_builder)
            .finish()
    }
}

impl<O, S, P> Default for Table<O, S, P>
where
    O: TableOps,
    O::Indices: Default,
    S: Default,
{
    fn default() -> Self {
        Self {
            global_param: 0,
            params: Box::default(),
            indices: <O::Indices as Default>::default(),
            entries: Box::default(),
            hasher_builder: S::default(),
            _portable: PhantomData,
        }
    }
}

impl<O, S, P> Clone for Table<O, S, P>
where
    O: TableOps,
    O::Indices: Clone,
    O::Entry: Clone,
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            global_param: self.global_param,
            params: self.params.clone(),
            indices: self.indices.clone(),
            entries: self.entries.clone(),
            hasher_builder: self.hasher_builder.clone(),
            _portable: PhantomData,
        }
    }

    fn clone_from(&mut self, other: &Self) {
        self.global_param = other.global_param;
        self.params.clone_from(&other.params);
        self.indices.clone_from(&other.indices);
        self.entries.clone_from(&other.entries);
        self.hasher_builder.clone_from(&other.hasher_builder);
    }
}

impl<O: TableOps, S, P> IntoIterator for Table<O, S, P> {
    type Item = O::Entry;
    type IntoIter = alloc::vec::IntoIter<O::Entry>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl<O: TableOps, S1, S2, P1, P2> PartialEq<Table<O, S2, P2>> for Table<O, S1, P1>
where
    O::Entry: PartialEq,
    O::Key: HashOps<S2, P2> + EqOps<O::Key, P2>,
    S1: BuildHasherOps<P1>,
    S2: BuildHasherOps<P2>,
    u32: HashOps<S2, P2>,
    u8: HashOps<S2, P2>,
{
    fn eq(&self, other: &Table<O, S2, P2>) -> bool {
        if self.len() != other.len() {
            return false;
        }

        for entry in self.iter() {
            let Some(other) = other.get(O::get_key(entry)) else {
                return false;
            };
            if entry != other {
                return false;
            }
        }

        true
    }
}

impl<O: TableOps, S, P> Eq for Table<O, S, P>
where
    O::Entry: Eq,
    O::Key: HashOps<S, P> + EqOps<O::Key, P>,
    S: BuildHasherOps<P>,
    u32: HashOps<S, P>,
    u8: HashOps<S, P>,
{
}

/// Whether no index appears twice. `None`s are ignored, being absent keys.
#[inline]
fn unique_indices(mut indices: &[Option<usize>]) -> bool {
    while let Some((&first, rest)) = indices.split_first() {
        if let Some(first) = first
            && rest.iter().filter_map(|idx| *idx).any(|idx| first == idx)
        {
            return false;
        }
        indices = rest;
    }
    true
}

#[cfg(feature = "serde")]
impl<O, H> serde::Serialize for Table<O, H, crate::portability::Portable>
where
    O: TableOps,
    O::Entry: serde::Serialize,
    O::Indices: serde::Serialize,
    H: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut ser =
            serializer.serialize_struct("Table", 5 - (if O::HAVE_INDICES { 0 } else { 1 }))?;
        ser.serialize_field("hasher", &self.hasher_builder)?;
        ser.serialize_field("entries", &self.entries)?;
        ser.serialize_field("global_param", &self.global_param)?;
        ser.serialize_field("params", &self.params)?;
        if O::HAVE_INDICES {
            ser.serialize_field("indices", &self.indices)?;
        }
        ser.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, O, S> serde::Deserialize<'de> for Table<O, S, crate::portability::Portable>
where
    O: TableOps,
    O::Entry: serde::Deserialize<'de>,
    O::Indices: serde::Deserialize<'de> + Default,
    S: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[allow(clippy::type_complexity)]
        struct Visitor<O, S>(core::marker::PhantomData<fn() -> (PhantomData<O>, S)>);

        enum Field {
            Hasher,
            Entries,
            GlobalParam,
            Params,
            Indices,
        }

        impl<'de> serde::Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct Visitor;

                impl serde::de::Visitor<'_> for Visitor {
                    type Value = Field;

                    fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                        f.write_str("one of \"global_param\", \"params\", \"indices\", \"entries\", or \"hasher\"")
                    }

                    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        match v {
                            0 => Ok(Field::Hasher),
                            1 => Ok(Field::Entries),
                            2 => Ok(Field::GlobalParam),
                            3 => Ok(Field::Params),
                            4 => Ok(Field::Indices),
                            _ => Err(E::invalid_value(serde::de::Unexpected::Unsigned(v), &self)),
                        }
                    }

                    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        match v {
                            0 => Ok(Field::Hasher),
                            1 => Ok(Field::Entries),
                            2 => Ok(Field::GlobalParam),
                            3 => Ok(Field::Params),
                            4 => Ok(Field::Indices),
                            _ => Err(E::invalid_value(serde::de::Unexpected::Signed(v), &self)),
                        }
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        match v {
                            "hasher" => Ok(Field::Hasher),
                            "entries" => Ok(Field::Entries),
                            "global_param" => Ok(Field::GlobalParam),
                            "params" => Ok(Field::Params),
                            "indices" => Ok(Field::Indices),
                            _ => Err(E::invalid_value(serde::de::Unexpected::Str(v), &self)),
                        }
                    }
                }

                deserializer.deserialize_identifier(Visitor)
            }
        }

        impl<'de, O, S> serde::de::Visitor<'de> for Visitor<O, S>
        where
            O: TableOps,
            O::Entry: serde::Deserialize<'de>,
            O::Indices: serde::Deserialize<'de> + Default,
            S: serde::Deserialize<'de>,
        {
            type Value = Table<O, S, crate::portability::Portable>;

            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("a phf table layout")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let hasher_builder: S = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::missing_field("hasher"))?;
                let entries: Box<[O::Entry]> = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::missing_field("entries"))?;
                let global_param: u8 = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::missing_field("global_param"))?;
                let params: Box<[u32]> = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::missing_field("params"))?;
                let indices: O::Indices = if O::HAVE_INDICES {
                    seq.next_element()?
                        .ok_or_else(|| serde::de::Error::missing_field("indices"))?
                } else {
                    <O::Indices as Default>::default()
                };

                let table = Table {
                    global_param,
                    params,
                    indices,
                    entries,
                    hasher_builder,
                    _portable: PhantomData,
                };

                if !O::verify(&table) {
                    return Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Other("phf table has invalid layout"),
                        &self,
                    ));
                }

                Ok(table)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut hasher_builder: Option<S> = None;
                let mut entries: Option<Box<[O::Entry]>> = None;
                let mut global_param: Option<u8> = None;
                let mut params: Option<Box<[u32]>> = None;
                let mut indices: Option<O::Indices> = None;

                while let Some(key) = map.next_key::<Field>()? {
                    match key {
                        Field::GlobalParam => {
                            if global_param.is_some() {
                                return Err(serde::de::Error::duplicate_field("global_param"));
                            }
                            global_param = Some(map.next_value()?);
                        }
                        Field::Params => {
                            if params.is_some() {
                                return Err(serde::de::Error::duplicate_field("params"));
                            }
                            params = Some(map.next_value()?);
                        }
                        Field::Indices if O::HAVE_INDICES => {
                            if indices.is_some() {
                                return Err(serde::de::Error::duplicate_field("indices"));
                            }
                            indices = Some(map.next_value()?);
                        }
                        Field::Indices => {
                            return Err(serde::de::Error::unknown_field(
                                "indices",
                                &["hasher", "entries", "global_param", "params"],
                            ));
                        }
                        Field::Entries => {
                            if entries.is_some() {
                                return Err(serde::de::Error::duplicate_field("entries"));
                            }
                            entries = Some(map.next_value()?);
                        }
                        Field::Hasher => {
                            if hasher_builder.is_some() {
                                return Err(serde::de::Error::duplicate_field("hasher"));
                            }
                            hasher_builder = Some(map.next_value()?);
                        }
                    }
                }

                let Some(hasher_builder) = hasher_builder else {
                    return Err(serde::de::Error::missing_field("hasher"));
                };
                let Some(entries) = entries else {
                    return Err(serde::de::Error::missing_field("entries"));
                };
                let Some(global_param) = global_param else {
                    return Err(serde::de::Error::missing_field("global_param"));
                };
                let Some(params) = params else {
                    return Err(serde::de::Error::missing_field("params"));
                };
                let indices = if O::HAVE_INDICES {
                    let Some(indices) = indices else {
                        return Err(serde::de::Error::missing_field("indices"));
                    };
                    indices
                } else {
                    <O::Indices as Default>::default()
                };

                let table = Table {
                    global_param,
                    params,
                    indices,
                    entries,
                    hasher_builder,
                    _portable: PhantomData,
                };

                if !O::verify(&table) {
                    return Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Other("phf table has invalid layout"),
                        &self,
                    ));
                }

                Ok(table)
            }
        }

        deserializer.deserialize_struct(
            "Table",
            if O::HAVE_INDICES {
                &["hasher", "entries", "global_param", "params", "indices"]
            } else {
                &["hasher", "entries", "global_param", "params"]
            },
            Visitor(core::marker::PhantomData),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portability::{MapOps, SetOps};
    use crate::table::Builder;

    use alloc::format;
    use alloc::vec::Vec;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    type SetTable = Table<SetOps<u32>, DefaultHasherSeed, NonPortable>;
    type MapTable = Table<MapOps<u32, u32>, DefaultHasherSeed, NonPortable>;

    fn set_of(keys: impl IntoIterator<Item = u32>) -> SetTable {
        let mut builder: Builder<SetOps<u32>, _, NonPortable> =
            Builder::with_hasher(DefaultHasherSeed::with_seed(1));
        for k in keys {
            builder.insert(k);
        }
        builder.build()
    }

    fn map_of(entries: impl IntoIterator<Item = (u32, u32)>) -> MapTable {
        let mut builder: Builder<MapOps<u32, u32>, _, NonPortable> =
            Builder::with_hasher(DefaultHasherSeed::with_seed(1));
        for (k, v) in entries {
            builder.map_insert(k, v);
        }
        builder.build()
    }

    #[test]
    fn an_empty_table_answers_every_query_without_touching_an_entry() {
        // Every lookup has to short-circuit here: the unchecked path divides by the entry count.
        let table = SetTable::default();

        assert!(table.is_empty());
        assert_eq!(table.len(), 0);
        assert_eq!(table.get_index(&1u32), None);
        assert!(!table.contains(&1u32));
        assert_eq!(table.get(&1u32), None);
        assert!(table.as_slice().is_empty());
        assert_eq!(table.index(0), None);

        let mut table = table;
        assert_eq!(table.get_mut(&1u32), None);
        assert_eq!(table.index_mut(0), None);
    }

    #[test]
    fn an_absent_key_still_resolves_to_an_entry_that_is_then_rejected() {
        let table = set_of(0..8);

        // The hash rounds always land somewhere, so the checked accessors are what turn a
        // missing key into `None`.
        for probe in [100u32, 101, 4242] {
            // SAFETY: the table is not empty.
            let index = unsafe { table.get_index_unchecked(&probe) };
            assert!(
                index < table.len(),
                "{index} out of bounds for an absent key"
            );

            assert_eq!(table.get_index(&probe), None);
            assert!(!table.contains(&probe));
            assert_eq!(table.get(&probe), None);
        }
    }

    #[test]
    fn entry_accessors_agree_with_their_unchecked_counterparts() {
        let mut table = set_of(0..8);

        for k in 0..8u32 {
            let index = table.get_index(&k).unwrap();
            assert_eq!(table.index(index), Some(&k));
            assert_eq!(table.get(&k), Some(&k));
            assert!(table.contains(&k));

            // SAFETY: `index` came from `get_index`, and the table is not empty.
            unsafe {
                assert_eq!(table.index_unchecked(index), &k);
                assert_eq!(table.get_unchecked(&k), &k);
            }
        }

        for k in 0..8u32 {
            let index = table.get_index(&k).unwrap();
            assert_eq!(table.index_mut(index), Some(&mut { k }));
            assert_eq!(table.get_mut(&k), Some(&mut { k }));
            // SAFETY: `index` came from `get_index`, and the table is not empty.
            unsafe {
                assert_eq!(table.index_unchecked_mut(index), &mut { k });
                assert_eq!(table.get_unchecked_mut(&k), &mut { k });
            }
        }
    }

    #[test]
    fn map_accessors_split_entries_into_keys_and_values() {
        let mut table = map_of((0..8).map(|k| (k, k * 10)));

        for k in 0..8u32 {
            let index = table.get_index(&k).unwrap();
            assert_eq!(table.map_index(index), Some((&k, &(k * 10))));
            assert_eq!(table.map_get_key_value(&k), Some((&k, &(k * 10))));
            assert_eq!(table.map_get(&k), Some(&(k * 10)));

            // SAFETY: `index` came from `get_index`, and the table is not empty.
            unsafe {
                assert_eq!(table.map_index_unchecked(index), (&k, &(k * 10)));
                assert_eq!(table.map_get_key_value_unchecked(&k), (&k, &(k * 10)));
                assert_eq!(table.map_get_unchecked(&k), &(k * 10));
            }
        }

        // The mutable halves reach the value only; the key comes back shared.
        let index = table.get_index(&3u32).unwrap();
        *table.map_index_mut(index).unwrap().1 = 1;
        *table.map_get_key_value_mut(&3u32).unwrap().1 += 1;
        *table.map_get_mut(&3u32).unwrap() += 1;
        // SAFETY: `index` came from `get_index`, and the table is not empty.
        unsafe {
            *table.map_index_unchecked_mut(index).1 += 1;
            *table.map_get_key_value_unchecked_mut(&3u32).1 += 1;
            *table.map_get_unchecked_mut(&3u32) += 1;
        }
        assert_eq!(table.map_get(&3u32), Some(&6));

        assert_eq!(table.map_get_key_value(&99u32), None);
        assert_eq!(table.map_get(&99u32), None);
        assert_eq!(table.map_index(99), None);
        assert_eq!(table.map_index_mut(99), None);
        assert_eq!(table.map_get_key_value_mut(&99u32), None);
        assert_eq!(table.map_get_mut(&99u32), None);
    }

    #[test]
    fn disjoint_accessors_hand_out_independent_borrows() {
        let mut table = map_of((0..8).map(|k| (k, k * 10)));

        let [a, b, missing] = table.map_get_disjoint_mut([&1u32, &2u32, &99u32]);
        *a.unwrap() = 111;
        *b.unwrap() = 222;
        assert!(missing.is_none());

        let [a, missing] = table.map_get_disjoint_key_value_mut([&3u32, &99u32]);
        let (key, value) = a.unwrap();
        assert_eq!(key, &3);
        *value = 333;
        assert!(missing.is_none());

        assert_eq!(table.map_get(&1u32), Some(&111));
        assert_eq!(table.map_get(&2u32), Some(&222));
        assert_eq!(table.map_get(&3u32), Some(&333));
    }

    #[test]
    fn unchecked_disjoint_accessors_hand_out_independent_borrows() {
        let mut table = map_of((0..8).map(|k| (k, k * 10)));

        // SAFETY: the keys are pairwise distinct, so they cannot share an entry.
        let [a, b, missing] =
            unsafe { table.map_get_disjoint_unchecked_mut([&1u32, &2u32, &99u32]) };
        *a.unwrap() = 111;
        *b.unwrap() = 222;
        assert!(missing.is_none());

        // SAFETY: the keys are pairwise distinct, so they cannot share an entry.
        let [a, missing] =
            unsafe { table.map_get_disjoint_key_value_unchecked_mut([&3u32, &99u32]) };
        *a.unwrap().1 = 333;
        assert!(missing.is_none());

        assert_eq!(table.map_get(&1u32), Some(&111));
        assert_eq!(table.map_get(&2u32), Some(&222));
        assert_eq!(table.map_get(&3u32), Some(&333));
    }

    #[test]
    fn the_checked_disjoint_accessors_refuse_to_alias() {
        let mut table = map_of((0..8).map(|k| (k, k * 10)));

        let aliased = catch_unwind(AssertUnwindSafe(|| {
            table.map_get_disjoint_mut([&1u32, &1u32]);
        }));
        assert!(
            aliased.is_err(),
            "two borrows of one entry must not be handed out"
        );

        let aliased = catch_unwind(AssertUnwindSafe(|| {
            table.map_get_disjoint_key_value_mut([&2u32, &2u32]);
        }));
        assert!(
            aliased.is_err(),
            "two borrows of one entry must not be handed out"
        );

        // Absent keys are `None` rather than duplicates, however many there are.
        let [x, y] = table.map_get_disjoint_mut([&98u32, &99u32]);
        assert!(x.is_none() && y.is_none());
    }

    #[test]
    fn unique_indices_ignores_absent_keys() {
        assert!(unique_indices(&[]));
        assert!(unique_indices(&[None, None]));
        assert!(unique_indices(&[Some(0), Some(1), None]));
        assert!(unique_indices(&[None, Some(2), None]));

        assert!(!unique_indices(&[Some(1), Some(1)]));
        assert!(!unique_indices(&[Some(0), None, Some(0)]));
    }

    #[test]
    fn iteration_visits_every_entry() {
        let mut table = map_of((0..5).map(|k| (k, k)));

        assert_eq!(table.iter().count(), 5);
        assert_eq!(table.as_slice().len(), 5);
        assert_eq!(table.hasher().seed(), 1);

        for (_, value) in table.iter_mut() {
            *value += 100;
        }
        for k in 0..5u32 {
            assert_eq!(table.map_get(&k), Some(&(k + 100)));
        }

        let mut owned = table.into_iter().collect::<Vec<_>>();
        owned.sort_unstable();
        assert_eq!(owned, (0..5).map(|k| (k, k + 100)).collect::<Vec<_>>());
    }

    #[test]
    fn a_table_shows_the_parameters_that_address_its_entries() {
        let shown = format!("{:?}", map_of([(1u32, 10u32)]));

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
    fn a_cloned_table_is_independent() {
        let table = map_of((0..5).map(|k| (k, k)));

        let mut clone = table.clone();
        *clone.map_get_mut(&1u32).unwrap() = 99;
        assert_eq!(table.map_get(&1u32), Some(&1));
        assert_eq!(clone.map_get(&1u32), Some(&99));

        clone.clone_from(&table);
        assert_eq!(clone.map_get(&1u32), Some(&1));
    }

    #[test]
    fn equality_ignores_the_hasher_and_the_entry_order() {
        // Different seeds permute the entries differently, so this compares by lookup rather
        // than position.
        let mut a: Builder<MapOps<u32, u32>, _, NonPortable> =
            Builder::with_hasher(DefaultHasherSeed::with_seed(1));
        let mut b: Builder<MapOps<u32, u32>, _, NonPortable> =
            Builder::with_hasher(DefaultHasherSeed::with_seed(2));
        for k in 0..16u32 {
            a.map_insert(k, k);
        }
        for k in (0..16u32).rev() {
            b.map_insert(k, k);
        }
        let (a, b) = (a.build(), b.build());

        assert_eq!(a, b);
        assert_eq!(a, a);
        assert_ne!(a, map_of((0..15).map(|k| (k, k))));
        assert_ne!(a, map_of((0..16).map(|k| (k, if k == 3 { 99 } else { k }))));
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use super::*;
    use crate::portability::{MapOps, OrderedMapOps, OrderedSetOps, Portable, SetOps};
    use crate::table::Builder;

    use alloc::string::{String, ToString};

    type OrderedSetTable = Table<OrderedSetOps<u32>, DefaultHasherSeed, Portable>;
    type UnorderedSetTable = Table<SetOps<u32>, DefaultHasherSeed, Portable>;

    fn build<O>(
        entries: impl IntoIterator<Item = O::Entry>,
    ) -> Builder<O, DefaultHasherSeed, Portable>
    where
        O: TableOps,
        O::Key: HashOps<DefaultHasherSeed, Portable> + EqOps<O::Key, Portable>,
    {
        let mut builder = Builder::with_hasher(DefaultHasherSeed::with_seed(7));
        for entry in entries {
            builder.insert(entry);
        }
        builder
    }

    /// The layout every case below starts from: three entries, an ordered table.
    const ORDERED: &str = r#"{"hasher":{"seed":7},"entries":[0,1,2],
        "global_param":0,"params":[0,1,0],"indices":[2,1,0]}"#;

    fn ordered_with(field: &str, value: &str) -> String {
        let original = serde_json::from_str::<serde_json::Value>(ORDERED).unwrap();
        let mut object = original.as_object().unwrap().clone();
        object.insert(field.into(), serde_json::from_str(value).unwrap());
        serde_json::to_string(&object).unwrap()
    }

    #[test]
    fn every_shape_survives_a_round_trip() {
        let ordered_set = build::<OrderedSetOps<u32>>(0..4u32).build();
        let set = build::<SetOps<u32>>(0..4u32).build();
        let ordered_map = build::<OrderedMapOps<u32, u32>>((0..4u32).map(|k| (k, k * 10))).build();
        let map = build::<MapOps<u32, u32>>((0..4u32).map(|k| (k, k * 10))).build();

        macro_rules! round_trip {
            ($table:expr, $ty:ty) => {{
                let json = serde_json::to_string(&$table).unwrap();
                let back: $ty = serde_json::from_str(&json).unwrap();
                assert_eq!(back, $table);
                back
            }};
        }

        let back = round_trip!(ordered_set, OrderedSetTable);
        assert_eq!(back.as_slice(), &[0, 1, 2, 3]);
        assert_eq!(back.hasher().seed(), 7);

        round_trip!(set, UnorderedSetTable);
        let back = round_trip!(
            ordered_map,
            Table<OrderedMapOps<u32, u32>, DefaultHasherSeed, Portable>
        );
        for k in 0..4u32 {
            assert_eq!(back.map_get(&k), Some(&(k * 10)));
        }
        round_trip!(map, Table<MapOps<u32, u32>, DefaultHasherSeed, Portable>);
    }

    #[test]
    fn the_slot_table_is_written_only_when_the_shape_has_one() {
        let ordered = serde_json::to_value(build::<OrderedSetOps<u32>>(0..3u32).build()).unwrap();
        let unordered = serde_json::to_value(build::<SetOps<u32>>(0..3u32).build()).unwrap();

        let fields = |v: &serde_json::Value| {
            let mut names = v
                .as_object()
                .unwrap()
                .keys()
                .cloned()
                .collect::<alloc::vec::Vec<_>>();
            names.sort();
            names
        };

        assert_eq!(
            fields(&ordered),
            ["entries", "global_param", "hasher", "indices", "params"],
        );
        assert_eq!(
            fields(&unordered),
            ["entries", "global_param", "hasher", "params"],
        );
    }

    #[test]
    fn a_slot_pointing_past_the_entries_is_refused() {
        // Accepting this would let an unchecked lookup read out of bounds.
        let tampered = ordered_with("indices", "[3,1,0]");
        assert!(serde_json::from_str::<OrderedSetTable>(&tampered).is_err());

        // The valid original is accepted, so the rejection is down to the slot alone.
        assert!(serde_json::from_str::<OrderedSetTable>(ORDERED).is_ok());
    }

    #[test]
    fn arrays_that_disagree_in_length_are_refused() {
        for (field, value) in [
            ("params", "[0,1]"),
            ("indices", "[1,0]"),
            ("entries", "[0,1]"),
        ] {
            let tampered = ordered_with(field, value);
            assert!(
                serde_json::from_str::<OrderedSetTable>(&tampered).is_err(),
                "a short `{field}` must be refused",
            );
        }
    }

    #[test]
    fn a_missing_field_is_refused() {
        for field in ["hasher", "entries", "global_param", "params", "indices"] {
            let mut object = serde_json::from_str::<serde_json::Value>(ORDERED)
                .unwrap()
                .as_object()
                .unwrap()
                .clone();
            object.remove(field);
            let json = serde_json::to_string(&object).unwrap();

            let error = serde_json::from_str::<OrderedSetTable>(&json)
                .unwrap_err()
                .to_string();
            assert!(
                error.contains(field),
                "dropping `{field}` should say so: {error}"
            );
        }
    }

    #[test]
    fn a_repeated_field_is_refused() {
        let json = r#"{"hasher":{"seed":7},"entries":[0,1,2],"entries":[0,1,2],
            "global_param":0,"params":[0,1,0],"indices":[2,1,0]}"#;

        let error = serde_json::from_str::<OrderedSetTable>(json)
            .unwrap_err()
            .to_string();
        assert!(error.contains("entries"), "{error}");
    }

    #[test]
    fn a_shape_without_a_slot_table_refuses_one() {
        // `indices` is a known field name, but not for this shape.
        let json = r#"{"hasher":{"seed":7},"entries":[2,1,0],
            "global_param":0,"params":[0,1,0],"indices":[2,1,0]}"#;

        let error = serde_json::from_str::<UnorderedSetTable>(json)
            .unwrap_err()
            .to_string();
        assert!(error.contains("indices"), "{error}");
        // The rejection has to name every field this shape does take, `global_param` included.
        for expected in ["hasher", "entries", "global_param", "params"] {
            assert!(
                error.contains(expected),
                "`{expected}` missing from: {error}"
            );
        }

        // Without it the same bytes deserialize.
        let json = r#"{"hasher":{"seed":7},"entries":[2,1,0],"global_param":0,"params":[0,1,0]}"#;
        assert!(serde_json::from_str::<UnorderedSetTable>(json).is_ok());
    }

    #[test]
    fn an_unrecognized_field_is_refused() {
        let json = ordered_with("bogus", "1");
        assert!(serde_json::from_str::<OrderedSetTable>(&json).is_err());
    }
}
