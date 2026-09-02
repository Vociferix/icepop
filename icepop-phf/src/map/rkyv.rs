//! The archived form of a [`PortableMap`](crate::PortableMap).

use super::Map;
use crate::portability::{MapOps, Portable};
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

/// A [`PortableMap`](crate::PortableMap) read in place, out of its serialized bytes.
///
/// Answers the same queries as the map it was archived from, with no deserialization and no
/// copies: the minimal perfect hash function is part of the archive, so a lookup reads the
/// buffer directly. Keys and values come back in their archived form, so a `u32` value is
/// returned as an [`rkyv::rend::u32_le`].
///
/// Obtain one with [`rkyv::access`] over a buffer from
/// [`rkyv::to_bytes`]. Validation checks the table's layout, so a corrupted
/// archive is rejected rather than mis-read.
///
/// Values can still be edited in place. Those methods are named `*_seal`, take the map as an
/// [`rkyv::seal::Seal`] from [`rkyv::access_mut`]
/// rather than a `&mut self`, and never hand out a mutable key.
///
/// # Example
///
/// ```
/// use icepop_phf::{PortableMap, map::rkyv::ArchivedMap};
/// use rkyv::rancor::Error;
///
/// let map: PortableMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
/// let bytes = rkyv::to_bytes::<Error>(&map)?;
/// let archived = rkyv::access::<ArchivedMap<String, u32>, Error>(&bytes)?;
///
/// assert_eq!(archived.get("a").map(|v| v.to_native()), Some(1));
/// assert!(archived.get("z").is_none());
/// # Ok::<(), Error>(())
/// ```
#[derive(rkyv::Portable, CheckBytes)]
#[bytecheck(crate = rkyv::bytecheck)]
#[repr(transparent)]
pub struct ArchivedMap<K, V, S = DefaultHasherSeed>
where
    K: Archive,
    V: Archive,
    S: Archive,
{
    table: Archived<Table<MapOps<K, V>, S, Portable>>,
}

/// Intermediate state produced while serializing a [`Map`], for
/// [`Archive::resolve`].
#[repr(transparent)]
pub struct MapResolver<K, V, S = DefaultHasherSeed>
where
    K: Archive,
    V: Archive,
    S: Archive,
{
    table: Resolver<Table<MapOps<K, V>, S, Portable>>,
}

impl<K, V, S> Archive for Map<K, V, S, Portable>
where
    K: Archive,
    V: Archive,
    S: Archive,
{
    type Archived = ArchivedMap<K, V, S>;
    type Resolver = MapResolver<K, V, S>;

    fn resolve(&self, resolver: Self::Resolver, out: rkyv::Place<Self::Archived>) {
        self.table
            // SAFETY: the archived type is `repr(transparent)` over the archived table, so a place
            // for one is a valid place for the other.
            .resolve(resolver.table, unsafe { out.cast_unchecked() });
    }
}

impl<K, V, S, Ser> Serialize<Ser> for Map<K, V, S, Portable>
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
            .map(|table| MapResolver { table })
    }
}

impl<K, V, S, D> Deserialize<Map<K, V, S, Portable>, D> for ArchivedMap<K, V, S>
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
    fn deserialize(&self, deserializer: &mut D) -> Result<Map<K, V, S, Portable>, D::Error> {
        Ok(Map {
            table: self.table.deserialize(deserializer)?,
        })
    }
}

impl<K, V, S> ArchivedMap<K, V, S>
where
    K: Archive,
    V: Archive,
    S: Archive,
{
    fn table_seal(this: Seal<'_, Self>) -> Seal<'_, ArchivedTable<MapOps<K, V>, S>> {
        // SAFETY: the archived type is `repr(transparent)` over its `table` field, so resealing
        // that field grants exactly the capability the original seal did.
        Seal::new(unsafe { &mut Seal::unseal_unchecked(this).table })
    }

    /// Returns the archived hasher the map was built with.
    ///
    /// The hasher travels inside the archive, which is what lets lookups reproduce the hashes
    /// computed when the map was built, on any machine.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{DefaultHasherSeed, PortableMap, map::rkyv::ArchivedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map =
    ///     PortableMap::<String, u32>::builder_with_hasher(DefaultHasherSeed::with_seed(42)).build();
    /// let bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access::<ArchivedMap<String, u32>, Error>(&bytes)?;
    ///
    /// assert_eq!(archived.hasher().seed(), 42);
    /// # Ok::<(), Error>(())
    /// ```
    pub fn hasher(&self) -> &Archived<S> {
        self.table.hasher()
    }

    /// Returns `true` if the archived map contains no entries.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableMap, map::rkyv::ArchivedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map = PortableMap::<String, u32>::builder().build();
    /// let bytes = rkyv::to_bytes::<Error>(&map)?;
    ///
    /// assert!(rkyv::access::<ArchivedMap<String, u32>, Error>(&bytes)?.is_empty());
    /// # Ok::<(), Error>(())
    /// ```
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// Returns the number of entries in the archived map.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableMap, map::rkyv::ArchivedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&map)?;
    ///
    /// assert_eq!(rkyv::access::<ArchivedMap<String, u32>, Error>(&bytes)?.len(), 1);
    /// # Ok::<(), Error>(())
    /// ```
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Returns an iterator over the archived entries, in an arbitrary order.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableMap, map::rkyv::ArchivedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access::<ArchivedMap<String, u32>, Error>(&bytes)?;
    ///
    /// let entries: Vec<_> = archived.iter().map(|(k, v)| (k.as_str(), v.to_native())).collect();
    /// assert_eq!(entries, [("a", 1)]);
    /// # Ok::<(), Error>(())
    /// ```
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            iter: self.table.iter(),
        }
    }

    /// Returns an iterator over the archived entries with editable values.
    ///
    /// Keys are yielded by shared reference: changing one would invalidate the hash function.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableMap, map::rkyv::ArchivedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let mut bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access_mut::<ArchivedMap<String, u32>, Error>(&mut bytes)?;
    ///
    /// for (_, value) in ArchivedMap::iter_seal(archived) {
    ///     *rkyv::seal::Seal::unseal(value) = 9.into();
    /// }
    ///
    /// let archived = rkyv::access::<ArchivedMap<String, u32>, Error>(&bytes)?;
    /// assert_eq!(archived.get("a").map(|v| v.to_native()), Some(9));
    /// # Ok::<(), Error>(())
    /// ```
    pub fn iter_seal(this: Seal<'_, Self>) -> IterSeal<'_, K, V> {
        IterSeal {
            // SAFETY: `iter_mut` requires that keys are never written. The iterator built from it
            // yields keys by shared reference and values sealed, so no key is reachable mutably.
            iter: unsafe { ArchivedTable::iter_mut(Self::table_seal(this)) },
        }
    }

    /// Returns an iterator over the archived keys, in an arbitrary order.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableMap, map::rkyv::ArchivedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access::<ArchivedMap<String, u32>, Error>(&bytes)?;
    ///
    /// assert_eq!(archived.keys().map(|k| k.as_str()).collect::<Vec<_>>(), ["a"]);
    /// # Ok::<(), Error>(())
    /// ```
    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys {
            iter: self.table.iter(),
        }
    }

    /// Returns an iterator over the archived values, in an arbitrary order.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableMap, map::rkyv::ArchivedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access::<ArchivedMap<String, u32>, Error>(&bytes)?;
    ///
    /// assert_eq!(archived.values().map(|v| v.to_native()).collect::<Vec<_>>(), [1]);
    /// # Ok::<(), Error>(())
    /// ```
    pub fn values(&self) -> Values<'_, K, V> {
        Values {
            iter: self.table.iter(),
        }
    }

    /// Returns an iterator over the archived values, each editable in place.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableMap, map::rkyv::ArchivedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let mut bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access_mut::<ArchivedMap<String, u32>, Error>(&mut bytes)?;
    ///
    /// for value in ArchivedMap::values_seal(archived) {
    ///     *rkyv::seal::Seal::unseal(value) = 9.into();
    /// }
    ///
    /// let archived = rkyv::access::<ArchivedMap<String, u32>, Error>(&bytes)?;
    /// assert_eq!(archived.get("a").map(|v| v.to_native()), Some(9));
    /// # Ok::<(), Error>(())
    /// ```
    pub fn values_seal(this: Seal<'_, Self>) -> ValuesSeal<'_, K, V> {
        ValuesSeal {
            // SAFETY: `iter_mut` requires that keys are never written. The iterator built from it
            // yields keys by shared reference and values sealed, so no key is reachable mutably.
            iter: unsafe { ArchivedTable::iter_mut(Self::table_seal(this)) },
        }
    }

    /// Borrows the archived entries as a contiguous slice of pairs, in an arbitrary order.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableMap, map::rkyv::ArchivedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access::<ArchivedMap<String, u32>, Error>(&bytes)?;
    ///
    /// assert_eq!(archived.as_slice().len(), 1);
    /// # Ok::<(), Error>(())
    /// ```
    pub fn as_slice(&self) -> &[Archived<(K, V)>] {
        self.table.as_slice()
    }

    /// Returns the archived entry at `index`, or `None` if it is out of bounds.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableMap, map::rkyv::ArchivedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access::<ArchivedMap<String, u32>, Error>(&bytes)?;
    ///
    /// assert_eq!(archived.index(0).map(|(k, v)| (k.as_str(), v.to_native())), Some(("a", 1)));
    /// assert!(archived.index(1).is_none());
    /// # Ok::<(), Error>(())
    /// ```
    pub fn index(&self, index: usize) -> Option<(&Archived<K>, &Archived<V>)> {
        self.table.map_index(index)
    }

    /// Returns the archived entry at `index` with an editable value.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableMap, map::rkyv::ArchivedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let mut bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access_mut::<ArchivedMap<String, u32>, Error>(&mut bytes)?;
    ///
    /// let (_, value) = ArchivedMap::index_seal(archived, 0).unwrap();
    /// *rkyv::seal::Seal::unseal(value) = 9.into();
    ///
    /// let archived = rkyv::access::<ArchivedMap<String, u32>, Error>(&bytes)?;
    /// assert_eq!(archived.get("a").map(|v| v.to_native()), Some(9));
    /// # Ok::<(), Error>(())
    /// ```
    pub fn index_seal(
        this: Seal<'_, Self>,
        index: usize,
    ) -> Option<(&Archived<K>, Seal<'_, Archived<V>>)> {
        ArchivedTable::map_index_seal(Self::table_seal(this), index)
    }

    /// Returns the archived entry at `index` without a bounds check.
    ///
    /// # Safety
    ///
    /// `index` must be less than [`len`](Self::len).
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableMap, map::rkyv::ArchivedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access::<ArchivedMap<String, u32>, Error>(&bytes)?;
    ///
    /// // SAFETY: the archived map has one entry, so index 0 is in bounds.
    /// let (key, value) = unsafe { archived.index_unchecked(0) };
    /// assert_eq!((key.as_str(), value.to_native()), ("a", 1));
    /// # Ok::<(), Error>(())
    /// ```
    pub unsafe fn index_unchecked(&self, index: usize) -> (&Archived<K>, &Archived<V>) {
        // SAFETY: `Table::map_index_unchecked` has this function's contract, which the caller
        // upheld.
        unsafe { self.table.map_index_unchecked(index) }
    }

    /// Returns the archived entry at `index` with an editable value, without a bounds check.
    ///
    /// # Safety
    ///
    /// `index` must be less than [`len`](Self::len).
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableMap, map::rkyv::ArchivedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let mut bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access_mut::<ArchivedMap<String, u32>, Error>(&mut bytes)?;
    ///
    /// // SAFETY: the archived map has one entry, so index 0 is in bounds.
    /// let (_, value) = unsafe { ArchivedMap::index_unchecked_seal(archived, 0) };
    /// *rkyv::seal::Seal::unseal(value) = 9.into();
    ///
    /// let archived = rkyv::access::<ArchivedMap<String, u32>, Error>(&bytes)?;
    /// assert_eq!(archived.get("a").map(|v| v.to_native()), Some(9));
    /// # Ok::<(), Error>(())
    /// ```
    pub unsafe fn index_unchecked_seal(
        this: Seal<'_, Self>,
        index: usize,
    ) -> (&Archived<K>, Seal<'_, Archived<V>>) {
        // SAFETY: `ArchivedTable::map_index_unchecked_seal` has this function's contract, which the
        // caller upheld.
        unsafe { ArchivedTable::map_index_unchecked_seal(Self::table_seal(this), index) }
    }
}

impl<K, V, S> ArchivedMap<K, V, S>
where
    K: Archive + PortableEq<Archived<K>>,
    V: Archive,
    S: Archive,
    Archived<S>: PortableBuildHasher,
{
    /// Returns the index `key` hashes to, without confirming that it is present.
    ///
    /// # Safety
    ///
    /// The archived map must not be empty.
    ///
    /// Looking up a key that is not present returns an arbitrary in-bounds index rather than
    /// failing.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableMap, map::rkyv::ArchivedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access::<ArchivedMap<String, u32>, Error>(&bytes)?;
    ///
    /// // SAFETY: the archived map is not empty.
    /// let index = unsafe { archived.get_index_unchecked("a") };
    ///
    /// assert_eq!(index, 0);
    /// # Ok::<(), Error>(())
    /// ```
    pub unsafe fn get_index_unchecked<Q>(&self, key: &Q) -> usize
    where
        Q: PortableHash + PortableEq<Archived<K>> + ?Sized,
    {
        // SAFETY: `Table::get_index_unchecked` has this function's contract, which the caller
        // upheld.
        unsafe { self.table.get_index_unchecked(key) }
    }

    /// Returns the index of the archived entry for `key`, or `None` if there is none.
    ///
    /// The index addresses [`as_slice`](Self::as_slice) and [`index`](Self::index).
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableMap, map::rkyv::ArchivedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access::<ArchivedMap<String, u32>, Error>(&bytes)?;
    ///
    /// assert_eq!(archived.get_index("a"), Some(0));
    /// assert_eq!(archived.get_index("z"), None);
    /// # Ok::<(), Error>(())
    /// ```
    pub fn get_index<Q>(&self, key: &Q) -> Option<usize>
    where
        Q: PortableHash + PortableEq<Archived<K>> + ?Sized,
    {
        self.table.get_index(key)
    }

    /// Returns `true` if the archived map contains an entry for `key`.
    ///
    /// The key is compared against the archived keys, so it need not itself be archived: a
    /// `&str` looks up an archived `String`.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableMap, map::rkyv::ArchivedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access::<ArchivedMap<String, u32>, Error>(&bytes)?;
    ///
    /// assert!(archived.contains_key("a"));
    /// assert!(!archived.contains_key("z"));
    /// # Ok::<(), Error>(())
    /// ```
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        Q: PortableHash + PortableEq<Archived<K>> + ?Sized,
    {
        self.table.contains(key)
    }

    /// Returns the archived key and its value, or `None` if `key` is not present.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableMap, map::rkyv::ArchivedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access::<ArchivedMap<String, u32>, Error>(&bytes)?;
    ///
    /// let entry = archived.get_key_value("a").map(|(k, v)| (k.as_str(), v.to_native()));
    /// assert_eq!(entry, Some(("a", 1)));
    /// # Ok::<(), Error>(())
    /// ```
    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&Archived<K>, &Archived<V>)>
    where
        Q: PortableHash + PortableEq<Archived<K>> + ?Sized,
    {
        self.table.map_get_key_value(key)
    }

    /// Returns the archived key and an editable value, or `None` if `key` is not present.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableMap, map::rkyv::ArchivedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let mut bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access_mut::<ArchivedMap<String, u32>, Error>(&mut bytes)?;
    ///
    /// let (key, value) = ArchivedMap::get_key_value_seal(archived, "a").unwrap();
    /// assert_eq!(key.as_str(), "a");
    /// *rkyv::seal::Seal::unseal(value) = 9.into();
    ///
    /// let archived = rkyv::access::<ArchivedMap<String, u32>, Error>(&bytes)?;
    /// assert_eq!(archived.get("a").map(|v| v.to_native()), Some(9));
    /// # Ok::<(), Error>(())
    /// ```
    pub fn get_key_value_seal<'a, Q>(
        this: Seal<'a, Self>,
        key: &Q,
    ) -> Option<(&'a Archived<K>, Seal<'a, Archived<V>>)>
    where
        Q: PortableHash + PortableEq<Archived<K>> + ?Sized,
    {
        ArchivedTable::map_get_key_value_seal(Self::table_seal(this), key)
    }

    /// Returns the archived value for `key`, or `None` if it is not present.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableMap, map::rkyv::ArchivedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access::<ArchivedMap<String, u32>, Error>(&bytes)?;
    ///
    /// assert_eq!(archived.get("a").map(|v| v.to_native()), Some(1));
    /// assert!(archived.get("z").is_none());
    /// # Ok::<(), Error>(())
    /// ```
    pub fn get<Q>(&self, key: &Q) -> Option<&Archived<V>>
    where
        Q: PortableHash + PortableEq<Archived<K>> + ?Sized,
    {
        self.table.map_get(key)
    }

    /// Returns an editable value for `key`, or `None` if it is not present.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableMap, map::rkyv::ArchivedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let mut bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access_mut::<ArchivedMap<String, u32>, Error>(&mut bytes)?;
    ///
    /// let value = ArchivedMap::get_seal(archived, "a").unwrap();
    /// *rkyv::seal::Seal::unseal(value) = 9.into();
    ///
    /// let archived = rkyv::access::<ArchivedMap<String, u32>, Error>(&bytes)?;
    /// assert_eq!(archived.get("a").map(|v| v.to_native()), Some(9));
    /// # Ok::<(), Error>(())
    /// ```
    pub fn get_seal<'a, Q>(this: Seal<'a, Self>, key: &Q) -> Option<Seal<'a, Archived<V>>>
    where
        Q: PortableHash + PortableEq<Archived<K>> + ?Sized,
    {
        ArchivedTable::map_get_seal(Self::table_seal(this), key)
    }

    /// Returns the archived entry `key` hashes to, without confirming that the key matches.
    ///
    /// # Safety
    ///
    /// The archived map must not be empty.
    ///
    /// Looking up a key that is not present returns an arbitrary entry rather than failing.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableMap, map::rkyv::ArchivedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access::<ArchivedMap<String, u32>, Error>(&bytes)?;
    ///
    /// // SAFETY: the archived map is not empty.
    /// let (key, value) = unsafe { archived.get_key_value_unchecked("a") };
    /// assert_eq!((key.as_str(), value.to_native()), ("a", 1));
    /// # Ok::<(), Error>(())
    /// ```
    pub unsafe fn get_key_value_unchecked<Q>(&self, key: &Q) -> (&Archived<K>, &Archived<V>)
    where
        Q: PortableHash + PortableEq<Archived<K>> + ?Sized,
    {
        // SAFETY: `Table::map_get_key_value_unchecked` has this function's contract, which the
        // caller upheld.
        unsafe { self.table.map_get_key_value_unchecked(key) }
    }

    /// Returns the archived entry `key` hashes to with an editable value, without confirming
    /// that the key matches.
    ///
    /// # Safety
    ///
    /// The archived map must not be empty.
    ///
    /// Looking up a key that is not present returns an arbitrary entry rather than failing.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableMap, map::rkyv::ArchivedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let mut bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access_mut::<ArchivedMap<String, u32>, Error>(&mut bytes)?;
    ///
    /// // SAFETY: the archived map is not empty.
    /// let (_, value) = unsafe { ArchivedMap::get_key_value_unchecked_seal(archived, "a") };
    /// *rkyv::seal::Seal::unseal(value) = 9.into();
    ///
    /// let archived = rkyv::access::<ArchivedMap<String, u32>, Error>(&bytes)?;
    /// assert_eq!(archived.get("a").map(|v| v.to_native()), Some(9));
    /// # Ok::<(), Error>(())
    /// ```
    pub unsafe fn get_key_value_unchecked_seal<'a, Q>(
        this: Seal<'a, Self>,
        key: &Q,
    ) -> (&'a Archived<K>, Seal<'a, Archived<V>>)
    where
        Q: PortableHash + PortableEq<Archived<K>> + ?Sized,
    {
        // SAFETY: `ArchivedTable::map_get_key_value_unchecked_seal` has this function's contract,
        // which the caller upheld.
        unsafe { ArchivedTable::map_get_key_value_unchecked_seal(Self::table_seal(this), key) }
    }

    /// Returns the archived value `key` hashes to, without confirming that the key matches.
    ///
    /// # Safety
    ///
    /// The archived map must not be empty.
    ///
    /// Looking up a key that is not present returns an arbitrary value rather than failing.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableMap, map::rkyv::ArchivedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access::<ArchivedMap<String, u32>, Error>(&bytes)?;
    ///
    /// // SAFETY: the archived map is not empty.
    /// assert_eq!(unsafe { archived.get_unchecked("a") }.to_native(), 1);
    /// # Ok::<(), Error>(())
    /// ```
    pub unsafe fn get_unchecked<Q>(&self, key: &Q) -> &Archived<V>
    where
        Q: PortableHash + PortableEq<Archived<K>> + ?Sized,
    {
        // SAFETY: `Table::map_get_unchecked` has this function's contract, which the caller upheld.
        unsafe { self.table.map_get_unchecked(key) }
    }

    /// Returns an editable value for the entry `key` hashes to, without confirming that the key
    /// matches.
    ///
    /// # Safety
    ///
    /// The archived map must not be empty.
    ///
    /// Looking up a key that is not present returns an arbitrary value rather than failing.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableMap, map::rkyv::ArchivedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let mut bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access_mut::<ArchivedMap<String, u32>, Error>(&mut bytes)?;
    ///
    /// // SAFETY: the archived map is not empty.
    /// let value = unsafe { ArchivedMap::get_unchecked_seal(archived, "a") };
    /// *rkyv::seal::Seal::unseal(value) = 9.into();
    ///
    /// let archived = rkyv::access::<ArchivedMap<String, u32>, Error>(&bytes)?;
    /// assert_eq!(archived.get("a").map(|v| v.to_native()), Some(9));
    /// # Ok::<(), Error>(())
    /// ```
    pub unsafe fn get_unchecked_seal<'a, Q>(this: Seal<'a, Self>, key: &Q) -> Seal<'a, Archived<V>>
    where
        Q: PortableHash + PortableEq<Archived<K>> + ?Sized,
    {
        // SAFETY: `ArchivedTable::map_get_unchecked_seal` has this function's contract, which the
        // caller upheld.
        unsafe { ArchivedTable::map_get_unchecked_seal(Self::table_seal(this), key) }
    }

    /// Returns `N` archived entries at once, each with an editable value.
    ///
    /// Editing several values at once is otherwise impossible, since each
    /// [`get_seal`](Self::get_seal) consumes the map's seal. Missing keys yield `None` in place.
    ///
    /// # Panics
    ///
    /// Panics if two keys refer to the same entry, which would alias the same value twice.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableMap, map::rkyv::ArchivedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableMap<String, u32> =
    ///     [("a".to_string(), 1u32), ("b".to_string(), 2)].into_iter().collect();
    /// let mut bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access_mut::<ArchivedMap<String, u32>, Error>(&mut bytes)?;
    ///
    /// let [a, b] = ArchivedMap::get_disjoint_key_value_seal(archived, ["a", "z"]);
    /// assert_eq!(a.unwrap().0.as_str(), "a");
    /// assert!(b.is_none());
    /// # Ok::<(), Error>(())
    /// ```
    #[allow(clippy::type_complexity)]
    pub fn get_disjoint_key_value_seal<'a, Q, const N: usize>(
        this: Seal<'a, Self>,
        keys: [&Q; N],
    ) -> [Option<(&'a Archived<K>, Seal<'a, Archived<V>>)>; N]
    where
        Q: PortableHash + PortableEq<Archived<K>> + ?Sized,
    {
        ArchivedTable::map_get_disjoint_key_value_seal(Self::table_seal(this), keys)
    }

    /// Returns `N` archived values at once, each editable in place.
    ///
    /// Missing keys yield `None` in place.
    ///
    /// # Panics
    ///
    /// Panics if two keys refer to the same entry, which would alias the same value twice.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableMap, map::rkyv::ArchivedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableMap<String, u32> =
    ///     [("a".to_string(), 1u32), ("b".to_string(), 2)].into_iter().collect();
    /// let mut bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access_mut::<ArchivedMap<String, u32>, Error>(&mut bytes)?;
    ///
    /// let [a, b] = ArchivedMap::get_disjoint_seal(archived, ["a", "b"]);
    /// *rkyv::seal::Seal::unseal(a.unwrap()) = 10.into();
    /// *rkyv::seal::Seal::unseal(b.unwrap()) = 20.into();
    ///
    /// let archived = rkyv::access::<ArchivedMap<String, u32>, Error>(&bytes)?;
    /// assert_eq!(archived.get("a").map(|v| v.to_native()), Some(10));
    /// # Ok::<(), Error>(())
    /// ```
    pub fn get_disjoint_seal<'a, Q, const N: usize>(
        this: Seal<'a, Self>,
        keys: [&Q; N],
    ) -> [Option<Seal<'a, Archived<V>>>; N]
    where
        Q: PortableHash + PortableEq<Archived<K>> + ?Sized,
    {
        ArchivedTable::map_get_disjoint_seal(Self::table_seal(this), keys)
    }

    /// [`get_disjoint_key_value_seal`](Self::get_disjoint_key_value_seal) without the
    /// distinctness check.
    ///
    /// # Safety
    ///
    /// No two keys may refer to the same entry.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableMap, map::rkyv::ArchivedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableMap<String, u32> =
    ///     [("a".to_string(), 1u32), ("b".to_string(), 2)].into_iter().collect();
    /// let mut bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access_mut::<ArchivedMap<String, u32>, Error>(&mut bytes)?;
    ///
    /// // SAFETY: "a" and "b" are distinct keys, so they cannot share an entry.
    /// let [a, b] = unsafe { ArchivedMap::get_disjoint_key_value_unchecked_seal(archived, ["a", "b"]) };
    /// *rkyv::seal::Seal::unseal(a.unwrap().1) = 10.into();
    /// *rkyv::seal::Seal::unseal(b.unwrap().1) = 20.into();
    ///
    /// let archived = rkyv::access::<ArchivedMap<String, u32>, Error>(&bytes)?;
    /// assert_eq!(archived.get("a").map(|v| v.to_native()), Some(10));
    /// # Ok::<(), Error>(())
    /// ```
    #[allow(clippy::type_complexity)]
    pub unsafe fn get_disjoint_key_value_unchecked_seal<'a, Q, const N: usize>(
        this: Seal<'a, Self>,
        keys: [&Q; N],
    ) -> [Option<(&'a Archived<K>, Seal<'a, Archived<V>>)>; N]
    where
        Q: PortableHash + PortableEq<Archived<K>> + ?Sized,
    {
        // SAFETY: `ArchivedTable::map_get_disjoint_key_value_unchecked_seal` has this function's
        // contract, which the caller upheld.
        unsafe {
            ArchivedTable::map_get_disjoint_key_value_unchecked_seal(Self::table_seal(this), keys)
        }
    }

    /// [`get_disjoint_seal`](Self::get_disjoint_seal) without the distinctness check.
    ///
    /// # Safety
    ///
    /// No two keys may refer to the same entry.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableMap, map::rkyv::ArchivedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableMap<String, u32> =
    ///     [("a".to_string(), 1u32), ("b".to_string(), 2)].into_iter().collect();
    /// let mut bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access_mut::<ArchivedMap<String, u32>, Error>(&mut bytes)?;
    ///
    /// // SAFETY: "a" and "b" are distinct keys, so they cannot share an entry.
    /// let [a, b] = unsafe { ArchivedMap::get_disjoint_unchecked_seal(archived, ["a", "b"]) };
    /// *rkyv::seal::Seal::unseal(a.unwrap()) = 10.into();
    /// *rkyv::seal::Seal::unseal(b.unwrap()) = 20.into();
    ///
    /// let archived = rkyv::access::<ArchivedMap<String, u32>, Error>(&bytes)?;
    /// assert_eq!(archived.get("a").map(|v| v.to_native()), Some(10));
    /// # Ok::<(), Error>(())
    /// ```
    pub unsafe fn get_disjoint_unchecked_seal<'a, Q, const N: usize>(
        this: Seal<'a, Self>,
        keys: [&Q; N],
    ) -> [Option<Seal<'a, Archived<V>>>; N]
    where
        Q: PortableHash + PortableEq<Archived<K>> + ?Sized,
    {
        // SAFETY: `ArchivedTable::map_get_disjoint_unchecked_seal` has this function's contract,
        // which the caller upheld.
        unsafe { ArchivedTable::map_get_disjoint_unchecked_seal(Self::table_seal(this), keys) }
    }
}

impl<K, V, S> core::fmt::Debug for ArchivedMap<K, V, S>
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

impl<'a, K, V, S> IntoIterator for &'a ArchivedMap<K, V, S>
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

impl<K, V, S> PartialEq for ArchivedMap<K, V, S>
where
    K: Archive + PortableEq<Archived<K>>,
    V: Archive,
    S: Archive,
    Archived<K>: PortableHash + PortableEq + PartialEq,
    Archived<V>: PartialEq,
    Archived<S>: PortableBuildHasher,
{
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter().all(|(key, value)| {
            other
                .get_key_value(key)
                .is_some_and(|(other_key, other_value)| other_key == key && other_value == value)
        })
    }
}

impl<K, V, S> Eq for ArchivedMap<K, V, S>
where
    K: Archive + PortableEq<Archived<K>>,
    V: Archive,
    S: Archive,
    Archived<K>: PortableHash + PortableEq + Eq,
    Archived<V>: Eq,
    Archived<S>: PortableBuildHasher,
{
}

#[cfg(feature = "serde")]
impl<K, V, S> serde::Serialize for ArchivedMap<K, V, S>
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
