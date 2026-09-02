//! The archived form of a [`PortableOrderedMap`](crate::PortableOrderedMap).

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
            // SAFETY: the archived type is `repr(transparent)` over the archived table, so a place
            // for one is a valid place for the other.
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
    /// use icepop_phf::{DefaultHasherSeed, PortableOrderedMap, ordered_map::rkyv::ArchivedOrderedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map =
    ///     PortableOrderedMap::<String, u32>::builder_with_hasher(DefaultHasherSeed::with_seed(42)).build();
    /// let bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access::<ArchivedOrderedMap<String, u32>, Error>(&bytes)?;
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
    /// use icepop_phf::{PortableOrderedMap, ordered_map::rkyv::ArchivedOrderedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map = PortableOrderedMap::<String, u32>::builder().build();
    /// let bytes = rkyv::to_bytes::<Error>(&map)?;
    ///
    /// assert!(rkyv::access::<ArchivedOrderedMap<String, u32>, Error>(&bytes)?.is_empty());
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
    /// use icepop_phf::{PortableOrderedMap, ordered_map::rkyv::ArchivedOrderedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableOrderedMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&map)?;
    ///
    /// assert_eq!(rkyv::access::<ArchivedOrderedMap<String, u32>, Error>(&bytes)?.len(), 1);
    /// # Ok::<(), Error>(())
    /// ```
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Returns an iterator over the archived entries, in insertion order.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableOrderedMap, ordered_map::rkyv::ArchivedOrderedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableOrderedMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access::<ArchivedOrderedMap<String, u32>, Error>(&bytes)?;
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
    /// use icepop_phf::{PortableOrderedMap, ordered_map::rkyv::ArchivedOrderedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableOrderedMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let mut bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access_mut::<ArchivedOrderedMap<String, u32>, Error>(&mut bytes)?;
    ///
    /// for (_, value) in ArchivedOrderedMap::iter_seal(archived) {
    ///     *rkyv::seal::Seal::unseal(value) = 9.into();
    /// }
    ///
    /// let archived = rkyv::access::<ArchivedOrderedMap<String, u32>, Error>(&bytes)?;
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

    /// Returns an iterator over the archived keys, in insertion order.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableOrderedMap, ordered_map::rkyv::ArchivedOrderedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableOrderedMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access::<ArchivedOrderedMap<String, u32>, Error>(&bytes)?;
    ///
    /// assert_eq!(archived.keys().map(|k| k.as_str()).collect::<Vec<_>>(), ["a"]);
    /// # Ok::<(), Error>(())
    /// ```
    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys {
            iter: self.table.iter(),
        }
    }

    /// Returns an iterator over the archived values, in insertion order.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableOrderedMap, ordered_map::rkyv::ArchivedOrderedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableOrderedMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access::<ArchivedOrderedMap<String, u32>, Error>(&bytes)?;
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
    /// use icepop_phf::{PortableOrderedMap, ordered_map::rkyv::ArchivedOrderedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableOrderedMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let mut bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access_mut::<ArchivedOrderedMap<String, u32>, Error>(&mut bytes)?;
    ///
    /// for value in ArchivedOrderedMap::values_seal(archived) {
    ///     *rkyv::seal::Seal::unseal(value) = 9.into();
    /// }
    ///
    /// let archived = rkyv::access::<ArchivedOrderedMap<String, u32>, Error>(&bytes)?;
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

    /// Borrows the archived entries as a contiguous slice of pairs, in insertion order.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableOrderedMap, ordered_map::rkyv::ArchivedOrderedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableOrderedMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access::<ArchivedOrderedMap<String, u32>, Error>(&bytes)?;
    ///
    /// assert_eq!(archived.as_slice().len(), 1);
    /// # Ok::<(), Error>(())
    /// ```
    pub fn as_slice(&self) -> &[Archived<(K, V)>] {
        self.table.as_slice()
    }

    /// Returns the archived entry at `index`, or `None` if it is out of bounds.
    ///
    /// Indices run over insertion order.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableOrderedMap, ordered_map::rkyv::ArchivedOrderedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableOrderedMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access::<ArchivedOrderedMap<String, u32>, Error>(&bytes)?;
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
    /// use icepop_phf::{PortableOrderedMap, ordered_map::rkyv::ArchivedOrderedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableOrderedMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let mut bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access_mut::<ArchivedOrderedMap<String, u32>, Error>(&mut bytes)?;
    ///
    /// let (_, value) = ArchivedOrderedMap::index_seal(archived, 0).unwrap();
    /// *rkyv::seal::Seal::unseal(value) = 9.into();
    ///
    /// let archived = rkyv::access::<ArchivedOrderedMap<String, u32>, Error>(&bytes)?;
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
    /// use icepop_phf::{PortableOrderedMap, ordered_map::rkyv::ArchivedOrderedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableOrderedMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access::<ArchivedOrderedMap<String, u32>, Error>(&bytes)?;
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
    /// use icepop_phf::{PortableOrderedMap, ordered_map::rkyv::ArchivedOrderedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableOrderedMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let mut bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access_mut::<ArchivedOrderedMap<String, u32>, Error>(&mut bytes)?;
    ///
    /// // SAFETY: the archived map has one entry, so index 0 is in bounds.
    /// let (_, value) = unsafe { ArchivedOrderedMap::index_unchecked_seal(archived, 0) };
    /// *rkyv::seal::Seal::unseal(value) = 9.into();
    ///
    /// let archived = rkyv::access::<ArchivedOrderedMap<String, u32>, Error>(&bytes)?;
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

impl<K, V, S> ArchivedOrderedMap<K, V, S>
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
    /// use icepop_phf::{PortableOrderedMap, ordered_map::rkyv::ArchivedOrderedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableOrderedMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access::<ArchivedOrderedMap<String, u32>, Error>(&bytes)?;
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
    /// use icepop_phf::{PortableOrderedMap, ordered_map::rkyv::ArchivedOrderedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableOrderedMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access::<ArchivedOrderedMap<String, u32>, Error>(&bytes)?;
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
    /// use icepop_phf::{PortableOrderedMap, ordered_map::rkyv::ArchivedOrderedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableOrderedMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access::<ArchivedOrderedMap<String, u32>, Error>(&bytes)?;
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
    /// use icepop_phf::{PortableOrderedMap, ordered_map::rkyv::ArchivedOrderedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableOrderedMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access::<ArchivedOrderedMap<String, u32>, Error>(&bytes)?;
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
    /// use icepop_phf::{PortableOrderedMap, ordered_map::rkyv::ArchivedOrderedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableOrderedMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let mut bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access_mut::<ArchivedOrderedMap<String, u32>, Error>(&mut bytes)?;
    ///
    /// let (key, value) = ArchivedOrderedMap::get_key_value_seal(archived, "a").unwrap();
    /// assert_eq!(key.as_str(), "a");
    /// *rkyv::seal::Seal::unseal(value) = 9.into();
    ///
    /// let archived = rkyv::access::<ArchivedOrderedMap<String, u32>, Error>(&bytes)?;
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
    /// use icepop_phf::{PortableOrderedMap, ordered_map::rkyv::ArchivedOrderedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableOrderedMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access::<ArchivedOrderedMap<String, u32>, Error>(&bytes)?;
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
    /// use icepop_phf::{PortableOrderedMap, ordered_map::rkyv::ArchivedOrderedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableOrderedMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let mut bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access_mut::<ArchivedOrderedMap<String, u32>, Error>(&mut bytes)?;
    ///
    /// let value = ArchivedOrderedMap::get_seal(archived, "a").unwrap();
    /// *rkyv::seal::Seal::unseal(value) = 9.into();
    ///
    /// let archived = rkyv::access::<ArchivedOrderedMap<String, u32>, Error>(&bytes)?;
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
    /// use icepop_phf::{PortableOrderedMap, ordered_map::rkyv::ArchivedOrderedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableOrderedMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access::<ArchivedOrderedMap<String, u32>, Error>(&bytes)?;
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
    /// use icepop_phf::{PortableOrderedMap, ordered_map::rkyv::ArchivedOrderedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableOrderedMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let mut bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access_mut::<ArchivedOrderedMap<String, u32>, Error>(&mut bytes)?;
    ///
    /// // SAFETY: the archived map is not empty.
    /// let (_, value) = unsafe { ArchivedOrderedMap::get_key_value_unchecked_seal(archived, "a") };
    /// *rkyv::seal::Seal::unseal(value) = 9.into();
    ///
    /// let archived = rkyv::access::<ArchivedOrderedMap<String, u32>, Error>(&bytes)?;
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
    /// use icepop_phf::{PortableOrderedMap, ordered_map::rkyv::ArchivedOrderedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableOrderedMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access::<ArchivedOrderedMap<String, u32>, Error>(&bytes)?;
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
    /// use icepop_phf::{PortableOrderedMap, ordered_map::rkyv::ArchivedOrderedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableOrderedMap<String, u32> = [("a".to_string(), 1u32)].into_iter().collect();
    /// let mut bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access_mut::<ArchivedOrderedMap<String, u32>, Error>(&mut bytes)?;
    ///
    /// // SAFETY: the archived map is not empty.
    /// let value = unsafe { ArchivedOrderedMap::get_unchecked_seal(archived, "a") };
    /// *rkyv::seal::Seal::unseal(value) = 9.into();
    ///
    /// let archived = rkyv::access::<ArchivedOrderedMap<String, u32>, Error>(&bytes)?;
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
    /// use icepop_phf::{PortableOrderedMap, ordered_map::rkyv::ArchivedOrderedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableOrderedMap<String, u32> =
    ///     [("a".to_string(), 1u32), ("b".to_string(), 2)].into_iter().collect();
    /// let mut bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access_mut::<ArchivedOrderedMap<String, u32>, Error>(&mut bytes)?;
    ///
    /// let [a, b] = ArchivedOrderedMap::get_disjoint_key_value_seal(archived, ["a", "z"]);
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
    /// use icepop_phf::{PortableOrderedMap, ordered_map::rkyv::ArchivedOrderedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableOrderedMap<String, u32> =
    ///     [("a".to_string(), 1u32), ("b".to_string(), 2)].into_iter().collect();
    /// let mut bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access_mut::<ArchivedOrderedMap<String, u32>, Error>(&mut bytes)?;
    ///
    /// let [a, b] = ArchivedOrderedMap::get_disjoint_seal(archived, ["a", "b"]);
    /// *rkyv::seal::Seal::unseal(a.unwrap()) = 10.into();
    /// *rkyv::seal::Seal::unseal(b.unwrap()) = 20.into();
    ///
    /// let archived = rkyv::access::<ArchivedOrderedMap<String, u32>, Error>(&bytes)?;
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
    /// use icepop_phf::{PortableOrderedMap, ordered_map::rkyv::ArchivedOrderedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableOrderedMap<String, u32> =
    ///     [("a".to_string(), 1u32), ("b".to_string(), 2)].into_iter().collect();
    /// let mut bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access_mut::<ArchivedOrderedMap<String, u32>, Error>(&mut bytes)?;
    ///
    /// // SAFETY: "a" and "b" are distinct keys, so they cannot share an entry.
    /// let [a, b] = unsafe { ArchivedOrderedMap::get_disjoint_key_value_unchecked_seal(archived, ["a", "b"]) };
    /// *rkyv::seal::Seal::unseal(a.unwrap().1) = 10.into();
    /// *rkyv::seal::Seal::unseal(b.unwrap().1) = 20.into();
    ///
    /// let archived = rkyv::access::<ArchivedOrderedMap<String, u32>, Error>(&bytes)?;
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
    /// use icepop_phf::{PortableOrderedMap, ordered_map::rkyv::ArchivedOrderedMap};
    /// use rkyv::rancor::Error;
    ///
    /// let map: PortableOrderedMap<String, u32> =
    ///     [("a".to_string(), 1u32), ("b".to_string(), 2)].into_iter().collect();
    /// let mut bytes = rkyv::to_bytes::<Error>(&map)?;
    /// let archived = rkyv::access_mut::<ArchivedOrderedMap<String, u32>, Error>(&mut bytes)?;
    ///
    /// // SAFETY: "a" and "b" are distinct keys, so they cannot share an entry.
    /// let [a, b] = unsafe { ArchivedOrderedMap::get_disjoint_unchecked_seal(archived, ["a", "b"]) };
    /// *rkyv::seal::Seal::unseal(a.unwrap()) = 10.into();
    /// *rkyv::seal::Seal::unseal(b.unwrap()) = 20.into();
    ///
    /// let archived = rkyv::access::<ArchivedOrderedMap<String, u32>, Error>(&bytes)?;
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

impl<K, V, S> PartialEq for ArchivedOrderedMap<K, V, S>
where
    K: Archive,
    V: Archive,
    S: Archive,
    Archived<K>: PartialEq,
    Archived<V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<K, V, S> Eq for ArchivedOrderedMap<K, V, S>
where
    K: Archive,
    V: Archive,
    S: Archive,
    Archived<K>: Eq,
    Archived<V>: Eq,
{
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PortableOrderedMap;

    use alloc::format;
    use alloc::vec::Vec;
    use rkyv::rancor::Error;
    use rkyv::seal::Seal;

    type Archived = ArchivedOrderedMap<u32, u32>;

    fn bytes_of(entries: impl IntoIterator<Item = (u32, u32)>) -> rkyv::util::AlignedVec {
        let mut builder =
            PortableOrderedMap::<u32, u32>::builder_with_hasher(DefaultHasherSeed::with_seed(1));
        for (k, v) in entries {
            builder.insert(k, v);
        }
        rkyv::to_bytes::<Error>(&builder.build()).unwrap()
    }

    fn sorted<T: Ord>(iter: impl IntoIterator<Item = T>) -> Vec<T> {
        let mut all = iter.into_iter().collect::<Vec<_>>();
        all.sort();
        all
    }

    #[test]
    fn an_archived_map_answers_every_read() {
        let bytes = bytes_of((0..8).map(|k| (k, k * 10)));
        let archived = rkyv::access::<Archived, Error>(&bytes).unwrap();

        assert_eq!(archived.len(), 8);
        assert!(!archived.is_empty());
        assert_eq!(archived.hasher().seed(), 1);
        assert_eq!(archived.as_slice().len(), 8);
        assert_eq!(archived.into_iter().count(), 8);
        assert_eq!(
            sorted(archived.keys().map(|k| k.to_native())),
            [0, 1, 2, 3, 4, 5, 6, 7]
        );
        assert_eq!(
            sorted(archived.values().map(|v| v.to_native())),
            [0, 10, 20, 30, 40, 50, 60, 70],
        );
        assert_eq!(
            sorted(archived.iter().map(|(k, v)| (k.to_native(), v.to_native()))),
            sorted((0..8u32).map(|k| (k, k * 10))),
        );

        for k in 0..8u32 {
            let index = archived.get_index(&k).unwrap();
            assert!(archived.contains_key(&k));
            assert_eq!(archived.get(&k).unwrap().to_native(), k * 10);
            let (key, value) = archived.get_key_value(&k).unwrap();
            assert_eq!((key.to_native(), value.to_native()), (k, k * 10));
            let (key, value) = archived.index(index).unwrap();
            assert_eq!((key.to_native(), value.to_native()), (k, k * 10));

            // SAFETY: the archive is not empty and `index` came from `get_index`.
            unsafe {
                assert_eq!(archived.get_index_unchecked(&k), index);
                assert_eq!(archived.get_unchecked(&k).to_native(), k * 10);
                let (key, value) = archived.get_key_value_unchecked(&k);
                assert_eq!((key.to_native(), value.to_native()), (k, k * 10));
                let (key, value) = archived.index_unchecked(index);
                assert_eq!((key.to_native(), value.to_native()), (k, k * 10));
            }
        }

        assert_eq!(archived.get_index(&99u32), None);
        assert!(!archived.contains_key(&99u32));
        assert!(archived.get(&99u32).is_none());
        assert!(archived.get_key_value(&99u32).is_none());
        assert!(archived.index(8).is_none());

        let shown = format!("{archived:?}");
        assert!(shown.starts_with('{') && shown.ends_with('}'), "{shown}");
    }

    #[test]
    fn every_sealed_accessor_rewrites_a_value_and_leaves_the_key_alone() {
        let mut bytes = bytes_of((0..8).map(|k| (k, k * 10)));

        macro_rules! with_seal {
            (|$archived:ident| $body:expr) => {{
                let $archived = rkyv::access_mut::<Archived, Error>(&mut bytes).unwrap();
                $body
            }};
        }

        with_seal!(|a| {
            let value = Archived::get_seal(a, &1u32).unwrap();
            *Seal::unseal(value) = 111.into();
        });
        with_seal!(|a| {
            let (key, value) = Archived::get_key_value_seal(a, &2u32).unwrap();
            assert_eq!(key.to_native(), 2);
            *Seal::unseal(value) = 222.into();
        });
        with_seal!(|a| {
            let index = a.get_index(&3u32).unwrap();
            let (key, value) = Archived::index_seal(a, index).unwrap();
            assert_eq!(key.to_native(), 3);
            *Seal::unseal(value) = 333.into();
        });
        with_seal!(|a| {
            // SAFETY: the archive is not empty.
            let value = unsafe { Archived::get_unchecked_seal(a, &4u32) };
            *Seal::unseal(value) = 444.into();
        });
        with_seal!(|a| {
            // SAFETY: the archive is not empty.
            let (_, value) = unsafe { Archived::get_key_value_unchecked_seal(a, &5u32) };
            *Seal::unseal(value) = 555.into();
        });
        with_seal!(|a| {
            let index = a.get_index(&6u32).unwrap();
            // SAFETY: `index` came from `get_index`, so it is in bounds.
            let (_, value) = unsafe { Archived::index_unchecked_seal(a, index) };
            *Seal::unseal(value) = 666.into();
        });
        with_seal!(|a| {
            for (key, value) in Archived::iter_seal(a) {
                if key.to_native() == 7 {
                    *Seal::unseal(value) = 777.into();
                }
            }
        });

        let archived = rkyv::access::<Archived, Error>(&bytes).unwrap();
        for k in 1..8u32 {
            assert_eq!(archived.get(&k).unwrap().to_native(), k * 111);
        }
        // Keys are handed out shared, so none of that could have moved an entry.
        assert_eq!(
            sorted(archived.keys().map(|k| k.to_native())),
            [0, 1, 2, 3, 4, 5, 6, 7]
        );

        let a = rkyv::access_mut::<Archived, Error>(&mut bytes).unwrap();
        for value in Archived::values_seal(a) {
            *Seal::unseal(value) = 0.into();
        }
        let archived = rkyv::access::<Archived, Error>(&bytes).unwrap();
        assert!(archived.values().all(|v| v.to_native() == 0));

        let a = rkyv::access_mut::<Archived, Error>(&mut bytes).unwrap();
        assert!(Archived::get_seal(a, &99u32).is_none());
        let a = rkyv::access_mut::<Archived, Error>(&mut bytes).unwrap();
        assert!(Archived::get_key_value_seal(a, &99u32).is_none());
        let a = rkyv::access_mut::<Archived, Error>(&mut bytes).unwrap();
        assert!(Archived::index_seal(a, 8).is_none());
    }

    #[test]
    fn disjoint_seals_borrow_several_values_at_once() {
        let mut bytes = bytes_of((0..8).map(|k| (k, k * 10)));

        {
            let a = rkyv::access_mut::<Archived, Error>(&mut bytes).unwrap();
            let [x, y, missing] = Archived::get_disjoint_seal(a, [&1u32, &2u32, &99u32]);
            *Seal::unseal(x.unwrap()) = 111.into();
            *Seal::unseal(y.unwrap()) = 222.into();
            assert!(missing.is_none());
        }
        {
            let a = rkyv::access_mut::<Archived, Error>(&mut bytes).unwrap();
            let [x, missing] = Archived::get_disjoint_key_value_seal(a, [&3u32, &99u32]);
            let (key, value) = x.unwrap();
            assert_eq!(key.to_native(), 3);
            *Seal::unseal(value) = 333.into();
            assert!(missing.is_none());
        }
        {
            let a = rkyv::access_mut::<Archived, Error>(&mut bytes).unwrap();
            // SAFETY: the keys are pairwise distinct, so they cannot share an entry.
            let [x, y] = unsafe { Archived::get_disjoint_unchecked_seal(a, [&4u32, &5u32]) };
            *Seal::unseal(x.unwrap()) = 444.into();
            *Seal::unseal(y.unwrap()) = 555.into();
        }
        {
            let a = rkyv::access_mut::<Archived, Error>(&mut bytes).unwrap();
            // SAFETY: the keys are pairwise distinct, so they cannot share an entry.
            let [x, y] =
                unsafe { Archived::get_disjoint_key_value_unchecked_seal(a, [&6u32, &7u32]) };
            *Seal::unseal(x.unwrap().1) = 666.into();
            *Seal::unseal(y.unwrap().1) = 777.into();
        }

        let archived = rkyv::access::<Archived, Error>(&bytes).unwrap();
        for k in 1..8u32 {
            assert_eq!(archived.get(&k).unwrap().to_native(), k * 111);
        }
    }

    #[test]
    fn an_archive_round_trips_back_to_a_live_map() {
        let bytes = bytes_of((0..8).map(|k| (k, k * 10)));
        let archived = rkyv::access::<Archived, Error>(&bytes).unwrap();

        let map: PortableOrderedMap<u32, u32> = rkyv::deserialize::<_, Error>(archived).unwrap();

        assert_eq!(map.len(), 8);
        assert_eq!(map.hasher().seed(), 1);
        for k in 0..8u32 {
            assert_eq!(map.get(&k), Some(&(k * 10)));
        }
    }

    #[test]
    fn archived_equality_compares_keys_and_values() {
        let a = bytes_of((0..8).map(|k| (k, k * 10)));
        let b = bytes_of((0..8).rev().map(|k| (k, k * 10)));
        let differing = bytes_of((0..8).map(|k| (k, if k == 3 { 99 } else { k * 10 })));

        let aa = rkyv::access::<Archived, Error>(&a).unwrap();
        let ab = rkyv::access::<Archived, Error>(&b).unwrap();
        let ad = rkyv::access::<Archived, Error>(&differing).unwrap();

        // An ordered map compares entry order too, so the reversed build differs.
        assert_ne!(aa, ab);
        assert_ne!(aa, ad);
    }
}
