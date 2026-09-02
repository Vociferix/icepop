//! The archived form of a [`PortableOrderedSet`](crate::PortableOrderedSet).

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
            // SAFETY: the archived type is `repr(transparent)` over the archived table, so a place
            // for one is a valid place for the other.
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
    /// Returns the archived hasher the set was built with.
    ///
    /// The hasher travels inside the archive, which is what lets lookups reproduce the hashes
    /// computed when the set was built, on any machine.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{DefaultHasherSeed, PortableOrderedSet, ordered_set::rkyv::ArchivedOrderedSet};
    /// use rkyv::rancor::Error;
    ///
    /// let set = PortableOrderedSet::<u32>::builder_with_hasher(DefaultHasherSeed::with_seed(42)).build();
    /// let bytes = rkyv::to_bytes::<Error>(&set)?;
    /// let archived = rkyv::access::<ArchivedOrderedSet<u32>, Error>(&bytes)?;
    ///
    /// assert_eq!(archived.hasher().seed(), 42);
    /// # Ok::<(), Error>(())
    /// ```
    pub fn hasher(&self) -> &Archived<S> {
        self.table.hasher()
    }

    /// Returns `true` if the archived set contains no elements.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableOrderedSet, ordered_set::rkyv::ArchivedOrderedSet};
    /// use rkyv::rancor::Error;
    ///
    /// let set = PortableOrderedSet::<u32>::builder().build();
    /// let bytes = rkyv::to_bytes::<Error>(&set)?;
    ///
    /// assert!(rkyv::access::<ArchivedOrderedSet<u32>, Error>(&bytes)?.is_empty());
    /// # Ok::<(), Error>(())
    /// ```
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// Returns the number of elements in the archived set.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableOrderedSet, ordered_set::rkyv::ArchivedOrderedSet};
    /// use rkyv::rancor::Error;
    ///
    /// let set: PortableOrderedSet<u32> = [1u32, 2, 3].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&set)?;
    ///
    /// assert_eq!(rkyv::access::<ArchivedOrderedSet<u32>, Error>(&bytes)?.len(), 3);
    /// # Ok::<(), Error>(())
    /// ```
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Returns an iterator over the archived elements, in insertion order.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableOrderedSet, ordered_set::rkyv::ArchivedOrderedSet};
    /// use rkyv::rancor::Error;
    ///
    /// let set: PortableOrderedSet<u32> = [3u32, 1, 2].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&set)?;
    /// let archived = rkyv::access::<ArchivedOrderedSet<u32>, Error>(&bytes)?;
    ///
    /// let elements: Vec<u32> = archived.iter().map(|e| e.to_native()).collect();
    /// assert_eq!(elements, [3, 1, 2]);
    /// # Ok::<(), Error>(())
    /// ```
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            iter: self.table.iter(),
        }
    }

    /// Borrows the archived elements as a contiguous slice, in insertion order.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableOrderedSet, ordered_set::rkyv::ArchivedOrderedSet};
    /// use rkyv::rancor::Error;
    ///
    /// let set: PortableOrderedSet<u32> = [1u32, 2, 3].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&set)?;
    /// let archived = rkyv::access::<ArchivedOrderedSet<u32>, Error>(&bytes)?;
    ///
    /// assert_eq!(archived.as_slice().len(), 3);
    /// # Ok::<(), Error>(())
    /// ```
    pub fn as_slice(&self) -> &[Archived<T>] {
        self.table.as_slice()
    }

    /// Returns the archived element at `index`, or `None` if it is out of bounds.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableOrderedSet, ordered_set::rkyv::ArchivedOrderedSet};
    /// use rkyv::rancor::Error;
    ///
    /// let set: PortableOrderedSet<u32> = [10u32, 20].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&set)?;
    /// let archived = rkyv::access::<ArchivedOrderedSet<u32>, Error>(&bytes)?;
    /// assert_eq!(archived.index(0).map(|e| e.to_native()), Some(10));
    /// assert_eq!(archived.index(1).map(|e| e.to_native()), Some(20));
    /// assert!(archived.index(2).is_none());
    /// # Ok::<(), Error>(())
    /// ```
    pub fn index(&self, index: usize) -> Option<&Archived<T>> {
        self.table.index(index)
    }

    /// Returns the archived element at `index` without a bounds check.
    ///
    /// # Safety
    ///
    /// `index` must be less than [`len`](Self::len).
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableOrderedSet, ordered_set::rkyv::ArchivedOrderedSet};
    /// use rkyv::rancor::Error;
    ///
    /// let set: PortableOrderedSet<u32> = [10u32, 20].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&set)?;
    /// let archived = rkyv::access::<ArchivedOrderedSet<u32>, Error>(&bytes)?;
    ///
    /// // SAFETY: the archived set has two elements, so index 1 is in bounds.
    /// assert_eq!(unsafe { archived.index_unchecked(1) }.to_native(), 20);
    /// # Ok::<(), Error>(())
    /// ```
    pub unsafe fn index_unchecked(&self, index: usize) -> &Archived<T> {
        // SAFETY: `Table::index_unchecked` has this function's contract, which the caller upheld.
        unsafe { self.table.index_unchecked(index) }
    }
}

impl<T, S> ArchivedOrderedSet<T, S>
where
    T: Archive + PortableEq<Archived<T>>,
    S: Archive,
    Archived<S>: PortableBuildHasher,
{
    /// Returns the index `key` hashes to, without confirming that it is present.
    ///
    /// # Safety
    ///
    /// The archived set must not be empty.
    ///
    /// Looking up an element that is not present returns an arbitrary in-bounds index rather
    /// than failing.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableOrderedSet, ordered_set::rkyv::ArchivedOrderedSet};
    /// use rkyv::rancor::Error;
    ///
    /// let set: PortableOrderedSet<u32> = [1u32, 2, 3].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&set)?;
    /// let archived = rkyv::access::<ArchivedOrderedSet<u32>, Error>(&bytes)?;
    ///
    /// // SAFETY: the archived set is not empty.
    /// let index = unsafe { archived.get_index_unchecked(&2u32) };
    ///
    /// assert_eq!(archived.index(index).map(|e| e.to_native()), Some(2));
    /// # Ok::<(), Error>(())
    /// ```
    pub unsafe fn get_index_unchecked<Q>(&self, key: &Q) -> usize
    where
        Q: PortableHash + PortableEq<Archived<T>> + ?Sized,
    {
        // SAFETY: `Table::get_index_unchecked` has this function's contract, which the caller
        // upheld.
        unsafe { self.table.get_index_unchecked(key) }
    }

    /// Returns the index of the archived element equal to `key`, or `None` if there is none.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableOrderedSet, ordered_set::rkyv::ArchivedOrderedSet};
    /// use rkyv::rancor::Error;
    ///
    /// let set: PortableOrderedSet<u32> = [1u32, 2, 3].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&set)?;
    /// let archived = rkyv::access::<ArchivedOrderedSet<u32>, Error>(&bytes)?;
    /// let index = archived.get_index(&2u32).unwrap();
    ///
    /// assert_eq!(archived.index(index).map(|e| e.to_native()), Some(2));
    /// assert_eq!(archived.get_index(&9u32), None);
    /// # Ok::<(), Error>(())
    /// ```
    pub fn get_index<Q>(&self, key: &Q) -> Option<usize>
    where
        Q: PortableHash + PortableEq<Archived<T>> + ?Sized,
    {
        self.table.get_index(key)
    }

    /// Returns `true` if the archived set contains an element equal to `key`.
    ///
    /// The key is compared against the archived elements, so it need not itself be archived: a
    /// `&str` looks up an archived `String`.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableOrderedSet, ordered_set::rkyv::ArchivedOrderedSet};
    /// use rkyv::rancor::Error;
    ///
    /// let set: PortableOrderedSet<String> = ["ash".to_string()].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&set)?;
    /// let archived = rkyv::access::<ArchivedOrderedSet<String>, Error>(&bytes)?;
    ///
    /// assert!(archived.contains("ash"));
    /// assert!(!archived.contains("oak"));
    /// # Ok::<(), Error>(())
    /// ```
    pub fn contains<Q>(&self, key: &Q) -> bool
    where
        Q: PortableHash + PortableEq<Archived<T>> + ?Sized,
    {
        self.table.contains(key)
    }

    /// Returns the archived element equal to `key`, or `None` if there is none.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableOrderedSet, ordered_set::rkyv::ArchivedOrderedSet};
    /// use rkyv::rancor::Error;
    ///
    /// let set: PortableOrderedSet<String> = ["ash".to_string()].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&set)?;
    /// let archived = rkyv::access::<ArchivedOrderedSet<String>, Error>(&bytes)?;
    ///
    /// assert_eq!(archived.get("ash").map(|e| e.as_str()), Some("ash"));
    /// assert!(archived.get("oak").is_none());
    /// # Ok::<(), Error>(())
    /// ```
    pub fn get<Q>(&self, key: &Q) -> Option<&Archived<T>>
    where
        Q: PortableHash + PortableEq<Archived<T>> + ?Sized,
    {
        self.table.get(key)
    }

    /// Returns the archived element `key` hashes to, without confirming that it is equal.
    ///
    /// # Safety
    ///
    /// The archived set must not be empty.
    ///
    /// Looking up an element that is not present returns an arbitrary element rather than
    /// failing.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableOrderedSet, ordered_set::rkyv::ArchivedOrderedSet};
    /// use rkyv::rancor::Error;
    ///
    /// let set: PortableOrderedSet<u32> = [1u32, 2, 3].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&set)?;
    /// let archived = rkyv::access::<ArchivedOrderedSet<u32>, Error>(&bytes)?;
    ///
    /// // SAFETY: the archived set is not empty.
    /// assert_eq!(unsafe { archived.get_unchecked(&2u32) }.to_native(), 2);
    /// # Ok::<(), Error>(())
    /// ```
    pub unsafe fn get_unchecked<Q>(&self, key: &Q) -> &Archived<T>
    where
        Q: PortableHash + PortableEq<Archived<T>> + ?Sized,
    {
        // SAFETY: `Table::get_unchecked` has this function's contract, which the caller upheld.
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

impl<T, S> PartialEq for ArchivedOrderedSet<T, S>
where
    T: Archive,
    S: Archive,
    Archived<T>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T, S> Eq for ArchivedOrderedSet<T, S>
where
    T: Archive,
    S: Archive,
    Archived<T>: Eq,
{
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PortableOrderedSet;

    use alloc::format;
    use alloc::vec::Vec;
    use rkyv::rancor::Error;

    const ORDER: [u32; 5] = [9, 3, 7, 1, 8];

    fn archived_bytes(keys: impl IntoIterator<Item = u32>) -> rkyv::util::AlignedVec {
        let set: PortableOrderedSet<u32> = keys.into_iter().collect();
        rkyv::to_bytes::<Error>(&set).unwrap()
    }

    #[test]
    fn an_archived_set_keeps_its_order_and_answers_every_query() {
        let bytes = archived_bytes(ORDER);
        let archived = rkyv::access::<ArchivedOrderedSet<u32>, Error>(&bytes).unwrap();

        assert_eq!(archived.len(), 5);
        assert!(!archived.is_empty());
        assert_eq!(archived.as_slice().len(), 5);
        assert_eq!(
            archived.iter().map(|e| e.to_native()).collect::<Vec<_>>(),
            ORDER,
        );
        assert_eq!(archived.into_iter().count(), 5);

        for (position, &k) in ORDER.iter().enumerate() {
            assert_eq!(archived.get_index(&k), Some(position));
            assert!(archived.contains(&k));
            assert_eq!(archived.get(&k).unwrap().to_native(), k);
            assert_eq!(archived.index(position).unwrap().to_native(), k);

            // SAFETY: the archive is not empty and `position` is below the length.
            unsafe {
                assert_eq!(archived.get_index_unchecked(&k), position);
                assert_eq!(archived.get_unchecked(&k).to_native(), k);
                assert_eq!(archived.index_unchecked(position).to_native(), k);
            }
        }

        assert_eq!(archived.get_index(&99u32), None);
        assert!(!archived.contains(&99u32));
        assert!(archived.get(&99u32).is_none());
        assert!(archived.index(5).is_none());

        let shown = format!("{archived:?}");
        assert!(shown.starts_with('{') && shown.ends_with('}'), "{shown}");
    }

    #[test]
    fn an_archive_round_trips_back_to_a_live_set() {
        let bytes = archived_bytes(ORDER);
        let archived = rkyv::access::<ArchivedOrderedSet<u32>, Error>(&bytes).unwrap();

        let set: PortableOrderedSet<u32> = rkyv::deserialize::<_, Error>(archived).unwrap();

        assert_eq!(set.as_slice(), &ORDER);
    }

    #[test]
    fn archived_equality_compares_the_element_order() {
        let a = archived_bytes([1u32, 2, 3]);
        let same = archived_bytes([1u32, 2, 3]);
        let reordered = archived_bytes([3u32, 2, 1]);

        let aa = rkyv::access::<ArchivedOrderedSet<u32>, Error>(&a).unwrap();
        let ab = rkyv::access::<ArchivedOrderedSet<u32>, Error>(&same).unwrap();
        let ac = rkyv::access::<ArchivedOrderedSet<u32>, Error>(&reordered).unwrap();

        assert_eq!(aa, ab);
        assert_ne!(aa, ac);
    }

    #[test]
    fn the_archived_hasher_is_the_one_the_set_was_built_with() {
        let mut builder =
            PortableOrderedSet::<u32>::builder_with_hasher(DefaultHasherSeed::with_seed(21));
        builder.insert(1);
        let bytes = rkyv::to_bytes::<Error>(&builder.build()).unwrap();

        let archived = rkyv::access::<ArchivedOrderedSet<u32>, Error>(&bytes).unwrap();
        assert_eq!(archived.hasher().seed(), 21);
    }
}
