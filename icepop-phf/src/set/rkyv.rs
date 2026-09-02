//! The archived form of a [`PortableSet`](crate::PortableSet).

use super::Set;
use crate::portability::{Portable, SetOps};
use crate::table::Table;

use portable::{DefaultHasherSeed, PortableBuildHasher, PortableEq, PortableHash};

use rkyv::{
    Archive, Archived, Deserialize, DeserializeUnsized, Resolver, Serialize, SerializeUnsized,
    bytecheck::CheckBytes,
    rancor::{Fallible, Source},
};

#[doc(inline)]
pub use crate::set_iters::ArchivedIter as Iter;

/// A [`PortableSet`](crate::PortableSet) read in place, out of its serialized bytes.
///
/// Answers the same queries as the set it was archived from, with no deserialization and no
/// copies: the minimal perfect hash function is part of the archive, so a lookup reads the
/// buffer directly. Elements come back in their archived form, so a `u32` element is
/// returned as an [`rkyv::rend::u32_le`].
///
/// Obtain one with [`rkyv::access`] over a buffer from
/// [`rkyv::to_bytes`]. Validation checks the table's layout, so a
/// corrupted archive is rejected rather than mis-read.
///
/// # Example
///
/// ```
/// use icepop_phf::{PortableSet, set::rkyv::ArchivedSet};
/// use rkyv::rancor::Error;
///
/// let set: PortableSet<u32> = [1u32, 2, 3].into_iter().collect();
/// let bytes = rkyv::to_bytes::<Error>(&set)?;
/// let archived = rkyv::access::<ArchivedSet<u32>, Error>(&bytes)?;
///
/// assert!(archived.contains(&2u32));
/// assert!(!archived.contains(&9u32));
/// assert_eq!(archived.len(), 3);
/// # Ok::<(), Error>(())
/// ```
#[derive(rkyv::Portable, CheckBytes)]
#[bytecheck(crate = rkyv::bytecheck)]
#[repr(transparent)]
pub struct ArchivedSet<T, S = DefaultHasherSeed>
where
    T: Archive,
    S: Archive,
{
    table: Archived<Table<SetOps<T>, S, Portable>>,
}

/// Intermediate state produced while serializing a [`Set`], for
/// [`Archive::resolve`].
#[repr(transparent)]
pub struct SetResolver<T, S = DefaultHasherSeed>
where
    T: Archive,
    S: Archive,
{
    table: Resolver<Table<SetOps<T>, S, Portable>>,
}

impl<T, S> Archive for Set<T, S, Portable>
where
    T: Archive,
    S: Archive,
{
    type Archived = ArchivedSet<T, S>;
    type Resolver = SetResolver<T, S>;

    fn resolve(&self, resolver: Self::Resolver, out: rkyv::Place<Self::Archived>) {
        self.table
            // SAFETY: the archived type is `repr(transparent)` over the archived table, so a place
            // for one is a valid place for the other.
            .resolve(resolver.table, unsafe { out.cast_unchecked() })
    }
}

impl<T, S, Ser> Serialize<Ser> for Set<T, S, Portable>
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
            .map(|table| SetResolver { table })
    }
}

impl<T, S, D> Deserialize<Set<T, S, Portable>, D> for ArchivedSet<T, S>
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
    fn deserialize(&self, deserializer: &mut D) -> Result<Set<T, S, Portable>, D::Error> {
        Ok(Set {
            table: self.table.deserialize(deserializer)?,
        })
    }
}

impl<T, S> ArchivedSet<T, S>
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
    /// use icepop_phf::{DefaultHasherSeed, PortableSet, set::rkyv::ArchivedSet};
    /// use rkyv::rancor::Error;
    ///
    /// let set = PortableSet::<u32>::builder_with_hasher(DefaultHasherSeed::with_seed(42)).build();
    /// let bytes = rkyv::to_bytes::<Error>(&set)?;
    /// let archived = rkyv::access::<ArchivedSet<u32>, Error>(&bytes)?;
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
    /// use icepop_phf::{PortableSet, set::rkyv::ArchivedSet};
    /// use rkyv::rancor::Error;
    ///
    /// let set = PortableSet::<u32>::builder().build();
    /// let bytes = rkyv::to_bytes::<Error>(&set)?;
    ///
    /// assert!(rkyv::access::<ArchivedSet<u32>, Error>(&bytes)?.is_empty());
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
    /// use icepop_phf::{PortableSet, set::rkyv::ArchivedSet};
    /// use rkyv::rancor::Error;
    ///
    /// let set: PortableSet<u32> = [1u32, 2, 3].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&set)?;
    ///
    /// assert_eq!(rkyv::access::<ArchivedSet<u32>, Error>(&bytes)?.len(), 3);
    /// # Ok::<(), Error>(())
    /// ```
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Returns an iterator over the archived elements, in an arbitrary order.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableSet, set::rkyv::ArchivedSet};
    /// use rkyv::rancor::Error;
    ///
    /// let set: PortableSet<u32> = [1u32, 2, 3].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&set)?;
    /// let archived = rkyv::access::<ArchivedSet<u32>, Error>(&bytes)?;
    ///
    /// let mut elements: Vec<u32> = archived.iter().map(|e| e.to_native()).collect();
    /// elements.sort();
    /// assert_eq!(elements, [1, 2, 3]);
    /// # Ok::<(), Error>(())
    /// ```
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            iter: self.table.iter(),
        }
    }

    /// Borrows the archived elements as a contiguous slice, in an arbitrary order.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{PortableSet, set::rkyv::ArchivedSet};
    /// use rkyv::rancor::Error;
    ///
    /// let set: PortableSet<u32> = [1u32, 2, 3].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&set)?;
    /// let archived = rkyv::access::<ArchivedSet<u32>, Error>(&bytes)?;
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
    /// use icepop_phf::{PortableSet, set::rkyv::ArchivedSet};
    /// use rkyv::rancor::Error;
    ///
    /// let set: PortableSet<u32> = [10u32, 20].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&set)?;
    /// let archived = rkyv::access::<ArchivedSet<u32>, Error>(&bytes)?;
    /// let index = archived.get_index(&20u32).unwrap();
    ///
    /// assert_eq!(archived.index(index).map(|e| e.to_native()), Some(20));
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
    /// use icepop_phf::{PortableSet, set::rkyv::ArchivedSet};
    /// use rkyv::rancor::Error;
    ///
    /// let set: PortableSet<u32> = [10u32, 20].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&set)?;
    /// let archived = rkyv::access::<ArchivedSet<u32>, Error>(&bytes)?;
    /// let index = archived.get_index(&20u32).unwrap();
    ///
    /// // SAFETY: `get_index` returned this index, so it is in bounds.
    /// assert_eq!(unsafe { archived.index_unchecked(index) }.to_native(), 20);
    /// # Ok::<(), Error>(())
    /// ```
    pub unsafe fn index_unchecked(&self, index: usize) -> &Archived<T> {
        // SAFETY: `Table::index_unchecked` has this function's contract, which the caller upheld.
        unsafe { self.table.index_unchecked(index) }
    }
}

impl<T, S> ArchivedSet<T, S>
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
    /// use icepop_phf::{PortableSet, set::rkyv::ArchivedSet};
    /// use rkyv::rancor::Error;
    ///
    /// let set: PortableSet<u32> = [1u32, 2, 3].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&set)?;
    /// let archived = rkyv::access::<ArchivedSet<u32>, Error>(&bytes)?;
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
    /// use icepop_phf::{PortableSet, set::rkyv::ArchivedSet};
    /// use rkyv::rancor::Error;
    ///
    /// let set: PortableSet<u32> = [1u32, 2, 3].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&set)?;
    /// let archived = rkyv::access::<ArchivedSet<u32>, Error>(&bytes)?;
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
    /// use icepop_phf::{PortableSet, set::rkyv::ArchivedSet};
    /// use rkyv::rancor::Error;
    ///
    /// let set: PortableSet<String> = ["ash".to_string()].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&set)?;
    /// let archived = rkyv::access::<ArchivedSet<String>, Error>(&bytes)?;
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
    /// use icepop_phf::{PortableSet, set::rkyv::ArchivedSet};
    /// use rkyv::rancor::Error;
    ///
    /// let set: PortableSet<String> = ["ash".to_string()].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&set)?;
    /// let archived = rkyv::access::<ArchivedSet<String>, Error>(&bytes)?;
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
    /// use icepop_phf::{PortableSet, set::rkyv::ArchivedSet};
    /// use rkyv::rancor::Error;
    ///
    /// let set: PortableSet<u32> = [1u32, 2, 3].into_iter().collect();
    /// let bytes = rkyv::to_bytes::<Error>(&set)?;
    /// let archived = rkyv::access::<ArchivedSet<u32>, Error>(&bytes)?;
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

impl<T, S> core::fmt::Debug for ArchivedSet<T, S>
where
    T: Archive,
    S: Archive,
    Archived<T>: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl<'a, T, S> IntoIterator for &'a ArchivedSet<T, S>
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

impl<T, S> PartialEq for ArchivedSet<T, S>
where
    T: Archive + PortableEq<Archived<T>>,
    S: Archive,
    Archived<T>: PortableHash + PortableEq + PartialEq,
    Archived<S>: PortableBuildHasher,
{
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter()
            .all(|entry| other.get(entry).is_some_and(|other| other == entry))
    }
}

impl<T, S> Eq for ArchivedSet<T, S>
where
    T: Archive + PortableEq<Archived<T>>,
    S: Archive,
    Archived<T>: PortableHash + PortableEq + Eq,
    Archived<S>: PortableBuildHasher,
{
}

#[cfg(feature = "serde")]
impl<T, S> serde::Serialize for ArchivedSet<T, S>
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
    use crate::PortableSet;

    use alloc::format;
    use alloc::vec::Vec;
    use rkyv::rancor::Error;

    fn archived_bytes(keys: impl IntoIterator<Item = u32>) -> rkyv::util::AlignedVec {
        let mut builder = PortableSet::<u32>::builder_with_hasher(DefaultHasherSeed::with_seed(1));
        for k in keys {
            builder.insert(k);
        }
        rkyv::to_bytes::<Error>(&builder.build()).unwrap()
    }

    #[test]
    fn an_archived_set_answers_every_query() {
        let bytes = archived_bytes(0..8);
        let archived = rkyv::access::<ArchivedSet<u32>, Error>(&bytes).unwrap();

        assert_eq!(archived.len(), 8);
        assert!(!archived.is_empty());
        assert_eq!(archived.hasher().seed(), 1);
        assert_eq!(archived.as_slice().len(), 8);

        let mut seen = archived.iter().map(|e| e.to_native()).collect::<Vec<_>>();
        seen.sort_unstable();
        assert_eq!(seen, [0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(archived.into_iter().count(), 8);

        for k in 0..8u32 {
            let index = archived.get_index(&k).unwrap();
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
        assert!(archived.index(8).is_none());

        let shown = format!("{archived:?}");
        assert!(shown.starts_with('{') && shown.ends_with('}'), "{shown}");
    }

    #[test]
    fn an_archive_round_trips_back_to_a_live_set() {
        let bytes = archived_bytes(0..8);
        let archived = rkyv::access::<ArchivedSet<u32>, Error>(&bytes).unwrap();

        let set: PortableSet<u32> = rkyv::deserialize::<_, Error>(archived).unwrap();

        assert_eq!(set.len(), 8);
        assert_eq!(set.hasher().seed(), 1);
        for k in 0..8u32 {
            assert!(set.contains(&k));
        }
    }

    #[test]
    fn archived_equality_ignores_the_element_order() {
        let a = archived_bytes(0..8);
        let b = archived_bytes((0..8).rev());
        let shorter = archived_bytes(0..7);

        let aa = rkyv::access::<ArchivedSet<u32>, Error>(&a).unwrap();
        let ab = rkyv::access::<ArchivedSet<u32>, Error>(&b).unwrap();
        let as_ = rkyv::access::<ArchivedSet<u32>, Error>(&shorter).unwrap();

        assert_eq!(aa, ab);
        assert_ne!(aa, as_);
    }
}
