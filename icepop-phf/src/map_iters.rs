//! Iterators shared by [`Map`](crate::Map) and [`OrderedMap`](crate::OrderedMap).

use core::fmt;

/// Borrowing iterator over a map's entries.
///
/// Created by [`Map::iter`](crate::Map::iter), [`OrderedMap::iter`](crate::OrderedMap::iter)
/// and the corresponding builders. Entries arrive in the collection's own order: insertion
/// order for an ordered map, an arbitrary order otherwise.
///
/// # Example
///
/// ```
/// use icepop_phf::OrderedMap;
///
/// let mut map: OrderedMap<&str, u32> = [("b", 2u32), ("a", 1)].into_iter().collect();
///
/// let entries: Vec<_> = map.iter().map(|(k, v)| (*k, *v)).collect();
/// assert_eq!(entries, [("b", 2), ("a", 1)]);
/// ```
pub struct Iter<'a, K, V> {
    pub(crate) iter: core::slice::Iter<'a, (K, V)>,
}

/// Borrowing iterator over a map's entries, with mutable values.
///
/// Keys are yielded by shared reference: changing one would invalidate the hash function.
///
/// # Example
///
/// ```
/// use icepop_phf::OrderedMap;
///
/// let mut map: OrderedMap<&str, u32> = [("b", 2u32), ("a", 1)].into_iter().collect();
///
/// for (_, value) in map.iter_mut() {
///     *value *= 10;
/// }
///
/// assert_eq!(map.get("a"), Some(&10));
/// ```
pub struct IterMut<'a, K, V> {
    pub(crate) iter: core::slice::IterMut<'a, (K, V)>,
}

/// Owning iterator over a map's entries.
///
/// Created by [`IntoIterator`] on a map or its builder.
///
/// # Example
///
/// ```
/// use icepop_phf::OrderedMap;
///
/// let mut map: OrderedMap<&str, u32> = [("b", 2u32), ("a", 1)].into_iter().collect();
///
/// assert_eq!(map.into_iter().collect::<Vec<_>>(), [("b", 2), ("a", 1)]);
/// ```
#[derive(Clone)]
pub struct IntoIter<K, V> {
    pub(crate) iter: alloc::vec::IntoIter<(K, V)>,
}

/// Borrowing iterator over a map's keys.
///
/// # Example
///
/// ```
/// use icepop_phf::OrderedMap;
///
/// let mut map: OrderedMap<&str, u32> = [("b", 2u32), ("a", 1)].into_iter().collect();
///
/// assert_eq!(map.keys().copied().collect::<Vec<_>>(), ["b", "a"]);
/// ```
pub struct Keys<'a, K, V> {
    pub(crate) iter: core::slice::Iter<'a, (K, V)>,
}

/// Owning iterator over a map's keys, dropping its values.
///
/// # Example
///
/// ```
/// use icepop_phf::OrderedMap;
///
/// let mut map: OrderedMap<&str, u32> = [("b", 2u32), ("a", 1)].into_iter().collect();
///
/// assert_eq!(map.into_keys().collect::<Vec<_>>(), ["b", "a"]);
/// ```
#[derive(Clone)]
pub struct IntoKeys<K, V> {
    pub(crate) iter: alloc::vec::IntoIter<(K, V)>,
}

/// Borrowing iterator over a map's values.
///
/// # Example
///
/// ```
/// use icepop_phf::OrderedMap;
///
/// let mut map: OrderedMap<&str, u32> = [("b", 2u32), ("a", 1)].into_iter().collect();
///
/// assert_eq!(map.values().copied().collect::<Vec<_>>(), [2, 1]);
/// ```
pub struct Values<'a, K, V> {
    pub(crate) iter: core::slice::Iter<'a, (K, V)>,
}

/// Borrowing iterator over a map's values, mutably.
///
/// # Example
///
/// ```
/// use icepop_phf::OrderedMap;
///
/// let mut map: OrderedMap<&str, u32> = [("b", 2u32), ("a", 1)].into_iter().collect();
///
/// for value in map.values_mut() {
///     *value *= 10;
/// }
///
/// assert_eq!(map.values().copied().collect::<Vec<_>>(), [20, 10]);
/// ```
pub struct ValuesMut<'a, K, V> {
    pub(crate) iter: core::slice::IterMut<'a, (K, V)>,
}

/// Owning iterator over a map's values, dropping its keys.
///
/// # Example
///
/// ```
/// use icepop_phf::OrderedMap;
///
/// let mut map: OrderedMap<&str, u32> = [("b", 2u32), ("a", 1)].into_iter().collect();
///
/// assert_eq!(map.into_values().collect::<Vec<_>>(), [2, 1]);
/// ```
#[derive(Clone)]
pub struct IntoValues<K, V> {
    pub(crate) iter: alloc::vec::IntoIter<(K, V)>,
}

/// Borrowing iterator over an archived map's entries.
///
/// Yields keys and values in their archived form.
///
/// # Example
///
/// ```
/// use icepop_phf::{PortableOrderedMap, rkyv::ArchivedOrderedMap};
/// use rkyv::rancor::Error;
///
/// let map: PortableOrderedMap<String, u32> =
///     [("b".to_string(), 2u32), ("a".to_string(), 1)].into_iter().collect();
/// let bytes = rkyv::to_bytes::<Error>(&map)?;
/// let archived = rkyv::access::<ArchivedOrderedMap<String, u32>, Error>(&bytes)?;
///
/// let entries: Vec<_> = archived.iter().map(|(k, v)| (k.as_str(), v.to_native())).collect();
/// assert_eq!(entries, [("b", 2), ("a", 1)]);
/// # Ok::<(), Error>(())
/// ```
#[cfg(feature = "rkyv")]
pub struct ArchivedIter<'a, K: rkyv::Archive, V: rkyv::Archive> {
    pub(crate) iter: core::slice::Iter<'a, rkyv::tuple::ArchivedTuple2<K::Archived, V::Archived>>,
}

/// Borrowing iterator over an archived map's entries, with editable values.
///
/// Keys are yielded by shared reference: changing one would invalidate the hash function.
///
/// # Example
///
/// ```
/// use icepop_phf::{PortableOrderedMap, rkyv::ArchivedOrderedMap};
/// use rkyv::rancor::Error;
///
/// let map: PortableOrderedMap<String, u32> =
///     [("b".to_string(), 2u32), ("a".to_string(), 1)].into_iter().collect();
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
#[cfg(feature = "rkyv")]
pub struct ArchivedIterSeal<'a, K: rkyv::Archive, V: rkyv::Archive> {
    pub(crate) iter:
        core::slice::IterMut<'a, rkyv::tuple::ArchivedTuple2<K::Archived, V::Archived>>,
}

/// Borrowing iterator over an archived map's keys.
///
/// # Example
///
/// ```
/// use icepop_phf::{PortableOrderedMap, rkyv::ArchivedOrderedMap};
/// use rkyv::rancor::Error;
///
/// let map: PortableOrderedMap<String, u32> =
///     [("b".to_string(), 2u32), ("a".to_string(), 1)].into_iter().collect();
/// let bytes = rkyv::to_bytes::<Error>(&map)?;
/// let archived = rkyv::access::<ArchivedOrderedMap<String, u32>, Error>(&bytes)?;
///
/// assert_eq!(archived.keys().map(|k| k.as_str()).collect::<Vec<_>>(), ["b", "a"]);
/// # Ok::<(), Error>(())
/// ```
#[cfg(feature = "rkyv")]
pub struct ArchivedKeys<'a, K: rkyv::Archive, V: rkyv::Archive> {
    pub(crate) iter: core::slice::Iter<'a, rkyv::tuple::ArchivedTuple2<K::Archived, V::Archived>>,
}

/// Borrowing iterator over an archived map's values.
///
/// # Example
///
/// ```
/// use icepop_phf::{PortableOrderedMap, rkyv::ArchivedOrderedMap};
/// use rkyv::rancor::Error;
///
/// let map: PortableOrderedMap<String, u32> =
///     [("b".to_string(), 2u32), ("a".to_string(), 1)].into_iter().collect();
/// let bytes = rkyv::to_bytes::<Error>(&map)?;
/// let archived = rkyv::access::<ArchivedOrderedMap<String, u32>, Error>(&bytes)?;
///
/// assert_eq!(archived.values().map(|v| v.to_native()).collect::<Vec<_>>(), [2, 1]);
/// # Ok::<(), Error>(())
/// ```
#[cfg(feature = "rkyv")]
pub struct ArchivedValues<'a, K: rkyv::Archive, V: rkyv::Archive> {
    pub(crate) iter: core::slice::Iter<'a, rkyv::tuple::ArchivedTuple2<K::Archived, V::Archived>>,
}

/// Borrowing iterator over an archived map's values, each editable in place.
///
/// # Example
///
/// ```
/// use icepop_phf::{PortableOrderedMap, rkyv::ArchivedOrderedMap};
/// use rkyv::rancor::Error;
///
/// let map: PortableOrderedMap<String, u32> =
///     [("b".to_string(), 2u32), ("a".to_string(), 1)].into_iter().collect();
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
#[cfg(feature = "rkyv")]
pub struct ArchivedValuesSeal<'a, K: rkyv::Archive, V: rkyv::Archive> {
    pub(crate) iter:
        core::slice::IterMut<'a, rkyv::tuple::ArchivedTuple2<K::Archived, V::Archived>>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(k, v)| (k, v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth(n).map(|(k, v)| (k, v))
    }

    fn last(self) -> Option<Self::Item> {
        self.iter.last().map(|(k, v)| (k, v))
    }

    fn count(self) -> usize {
        self.iter.count()
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.fold(init, move |agg, (k, v)| f(agg, (k, v)))
    }
}

impl<'a, K, V> DoubleEndedIterator for Iter<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|(k, v)| (k, v))
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth_back(n).map(|(k, v)| (k, v))
    }

    fn rfold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.rfold(init, move |agg, (k, v)| f(agg, (k, v)))
    }
}

impl<'a, K, V> ExactSizeIterator for Iter<'a, K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, K, V> core::iter::FusedIterator for Iter<'a, K, V> {}

impl<'a, K, V> Clone for Iter<'a, K, V> {
    fn clone(&self) -> Self {
        Iter {
            iter: self.iter.clone(),
        }
    }
}

impl<'a, K, V> fmt::Debug for Iter<'a, K, V>
where
    K: fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter.as_slice()).finish()
    }
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(k, v)| (&*k, v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth(n).map(|(k, v)| (&*k, v))
    }

    fn last(self) -> Option<Self::Item> {
        self.iter.last().map(|(k, v)| (&*k, v))
    }

    fn count(self) -> usize {
        self.iter.count()
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.fold(init, move |agg, (k, v)| f(agg, (k, v)))
    }
}

impl<'a, K, V> DoubleEndedIterator for IterMut<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|(k, v)| (&*k, v))
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth_back(n).map(|(k, v)| (&*k, v))
    }

    fn rfold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.rfold(init, move |agg, (k, v)| f(agg, (k, v)))
    }
}

impl<'a, K, V> ExactSizeIterator for IterMut<'a, K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, K, V> core::iter::FusedIterator for IterMut<'a, K, V> {}

impl<'a, K, V> fmt::Debug for IterMut<'a, K, V>
where
    K: fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter.as_slice()).finish()
    }
}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(k, _)| k)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth(n).map(|(k, _)| k)
    }

    fn last(self) -> Option<Self::Item> {
        self.iter.last().map(|(k, _)| k)
    }

    fn count(self) -> usize {
        self.iter.count()
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.fold(init, move |agg, (k, _)| f(agg, k))
    }
}

impl<'a, K, V> DoubleEndedIterator for Keys<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|(k, _)| k)
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth_back(n).map(|(k, _)| k)
    }

    fn rfold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.rfold(init, move |agg, (k, _)| f(agg, k))
    }
}

impl<'a, K, V> ExactSizeIterator for Keys<'a, K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, K, V> core::iter::FusedIterator for Keys<'a, K, V> {}

impl<'a, K, V> Clone for Keys<'a, K, V> {
    fn clone(&self) -> Self {
        Keys {
            iter: self.iter.clone(),
        }
    }
}

impl<'a, K, V> fmt::Debug for Keys<'a, K, V>
where
    K: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.iter.as_slice().iter().map(|(k, _)| k))
            .finish()
    }
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(_, v)| v)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth(n).map(|(_, v)| v)
    }

    fn last(self) -> Option<Self::Item> {
        self.iter.last().map(|(_, v)| v)
    }

    fn count(self) -> usize {
        self.iter.count()
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.fold(init, move |agg, (_, v)| f(agg, v))
    }
}

impl<'a, K, V> DoubleEndedIterator for Values<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|(_, v)| v)
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth_back(n).map(|(_, v)| v)
    }

    fn rfold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.rfold(init, move |agg, (_, v)| f(agg, v))
    }
}

impl<'a, K, V> ExactSizeIterator for Values<'a, K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, K, V> core::iter::FusedIterator for Values<'a, K, V> {}

impl<'a, K, V> Clone for Values<'a, K, V> {
    fn clone(&self) -> Self {
        Values {
            iter: self.iter.clone(),
        }
    }
}

impl<'a, K, V> fmt::Debug for Values<'a, K, V>
where
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.iter.as_slice().iter().map(|(_, v)| v))
            .finish()
    }
}

impl<'a, K, V> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(_, v)| v)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth(n).map(|(_, v)| v)
    }

    fn last(self) -> Option<Self::Item> {
        self.iter.last().map(|(_, v)| v)
    }

    fn count(self) -> usize {
        self.iter.count()
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.fold(init, move |agg, (_, v)| f(agg, v))
    }
}

impl<'a, K, V> DoubleEndedIterator for ValuesMut<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|(_, v)| v)
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth_back(n).map(|(_, v)| v)
    }

    fn rfold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.rfold(init, move |agg, (_, v)| f(agg, v))
    }
}

impl<'a, K, V> ExactSizeIterator for ValuesMut<'a, K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, K, V> core::iter::FusedIterator for ValuesMut<'a, K, V> {}

impl<'a, K, V> fmt::Debug for ValuesMut<'a, K, V>
where
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.iter.as_slice().iter().map(|(_, v)| v))
            .finish()
    }
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth(n)
    }

    fn last(self) -> Option<Self::Item> {
        self.iter.last()
    }

    fn count(self) -> usize {
        self.iter.count()
    }

    fn fold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.fold(init, f)
    }
}

impl<K, V> DoubleEndedIterator for IntoIter<K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth_back(n)
    }

    fn rfold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.rfold(init, f)
    }
}

impl<K, V> ExactSizeIterator for IntoIter<K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<K, V> core::iter::FusedIterator for IntoIter<K, V> {}

impl<K, V> fmt::Debug for IntoIter<K, V>
where
    K: fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter.as_slice()).finish()
    }
}

impl<K, V> Iterator for IntoKeys<K, V> {
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(k, _)| k)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth(n).map(|(k, _)| k)
    }

    fn last(self) -> Option<Self::Item> {
        self.iter.last().map(|(k, _)| k)
    }

    fn count(self) -> usize {
        self.iter.count()
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.fold(init, move |agg, (k, _)| f(agg, k))
    }
}

impl<K, V> DoubleEndedIterator for IntoKeys<K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|(k, _)| k)
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth_back(n).map(|(k, _)| k)
    }

    fn rfold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.rfold(init, move |agg, (k, _)| f(agg, k))
    }
}

impl<K, V> ExactSizeIterator for IntoKeys<K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<K, V> core::iter::FusedIterator for IntoKeys<K, V> {}

impl<K, V> fmt::Debug for IntoKeys<K, V>
where
    K: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.iter.as_slice().iter().map(|(k, _)| k))
            .finish()
    }
}

impl<K, V> Iterator for IntoValues<K, V> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(_, v)| v)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth(n).map(|(_, v)| v)
    }

    fn last(self) -> Option<Self::Item> {
        self.iter.last().map(|(_, v)| v)
    }

    fn count(self) -> usize {
        self.iter.count()
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.fold(init, move |agg, (_, v)| f(agg, v))
    }
}

impl<K, V> DoubleEndedIterator for IntoValues<K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|(_, v)| v)
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth_back(n).map(|(_, v)| v)
    }

    fn rfold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.rfold(init, move |agg, (_, v)| f(agg, v))
    }
}

impl<K, V> ExactSizeIterator for IntoValues<K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<K, V> core::iter::FusedIterator for IntoValues<K, V> {}

impl<K, V> fmt::Debug for IntoValues<K, V>
where
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.iter.as_slice().iter().map(|(_, v)| v))
            .finish()
    }
}

#[cfg(feature = "rkyv")]
impl<'a, K: rkyv::Archive, V: rkyv::Archive> Iterator for ArchivedIter<'a, K, V> {
    type Item = (&'a K::Archived, &'a V::Archived);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|entry| (&entry.0, &entry.1))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth(n).map(|entry| (&entry.0, &entry.1))
    }

    fn last(self) -> Option<Self::Item> {
        self.iter.last().map(|entry| (&entry.0, &entry.1))
    }

    fn count(self) -> usize {
        self.iter.count()
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter
            .fold(init, move |agg, entry| f(agg, (&entry.0, &entry.1)))
    }
}

#[cfg(feature = "rkyv")]
impl<K: rkyv::Archive, V: rkyv::Archive> DoubleEndedIterator for ArchivedIter<'_, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|entry| (&entry.0, &entry.1))
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth_back(n).map(|entry| (&entry.0, &entry.1))
    }

    fn rfold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter
            .rfold(init, move |agg, entry| f(agg, (&entry.0, &entry.1)))
    }
}

#[cfg(feature = "rkyv")]
impl<K: rkyv::Archive, V: rkyv::Archive> ExactSizeIterator for ArchivedIter<'_, K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

#[cfg(feature = "rkyv")]
impl<K: rkyv::Archive, V: rkyv::Archive> core::iter::FusedIterator for ArchivedIter<'_, K, V> {}

#[cfg(feature = "rkyv")]
impl<K: rkyv::Archive, V: rkyv::Archive> Clone for ArchivedIter<'_, K, V> {
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}

#[cfg(feature = "rkyv")]
impl<K: rkyv::Archive, V: rkyv::Archive> fmt::Debug for ArchivedIter<'_, K, V>
where
    K::Archived: fmt::Debug,
    V::Archived: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter.as_slice()).finish()
    }
}

#[cfg(feature = "rkyv")]
impl<'a, K: rkyv::Archive, V: rkyv::Archive> Iterator for ArchivedIterSeal<'a, K, V> {
    type Item = (&'a K::Archived, rkyv::seal::Seal<'a, V::Archived>);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|entry| (&entry.0, rkyv::seal::Seal::new(&mut entry.1)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.iter
            .nth(n)
            .map(|entry| (&entry.0, rkyv::seal::Seal::new(&mut entry.1)))
    }

    fn last(self) -> Option<Self::Item> {
        self.iter
            .last()
            .map(|entry| (&entry.0, rkyv::seal::Seal::new(&mut entry.1)))
    }

    fn count(self) -> usize {
        self.iter.count()
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.fold(init, move |agg, entry| {
            f(agg, (&entry.0, rkyv::seal::Seal::new(&mut entry.1)))
        })
    }
}

#[cfg(feature = "rkyv")]
impl<K: rkyv::Archive, V: rkyv::Archive> DoubleEndedIterator for ArchivedIterSeal<'_, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter
            .next_back()
            .map(|entry| (&entry.0, rkyv::seal::Seal::new(&mut entry.1)))
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter
            .nth_back(n)
            .map(|entry| (&entry.0, rkyv::seal::Seal::new(&mut entry.1)))
    }

    fn rfold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.rfold(init, move |agg, entry| {
            f(agg, (&entry.0, rkyv::seal::Seal::new(&mut entry.1)))
        })
    }
}

#[cfg(feature = "rkyv")]
impl<K: rkyv::Archive, V: rkyv::Archive> ExactSizeIterator for ArchivedIterSeal<'_, K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

#[cfg(feature = "rkyv")]
impl<K: rkyv::Archive, V: rkyv::Archive> core::iter::FusedIterator for ArchivedIterSeal<'_, K, V> {}

#[cfg(feature = "rkyv")]
impl<K: rkyv::Archive, V: rkyv::Archive> fmt::Debug for ArchivedIterSeal<'_, K, V>
where
    K::Archived: fmt::Debug,
    V::Archived: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter.as_slice()).finish()
    }
}

#[cfg(feature = "rkyv")]
impl<'a, K: rkyv::Archive, V: rkyv::Archive> Iterator for ArchivedKeys<'a, K, V> {
    type Item = &'a K::Archived;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|entry| &entry.0)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth(n).map(|entry| &entry.0)
    }

    fn last(self) -> Option<Self::Item> {
        self.iter.last().map(|entry| &entry.0)
    }

    fn count(self) -> usize {
        self.iter.count()
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.fold(init, move |agg, entry| f(agg, &entry.0))
    }
}

#[cfg(feature = "rkyv")]
impl<K: rkyv::Archive, V: rkyv::Archive> DoubleEndedIterator for ArchivedKeys<'_, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|entry| &entry.0)
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth_back(n).map(|entry| &entry.0)
    }

    fn rfold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.rfold(init, move |agg, entry| f(agg, &entry.0))
    }
}

#[cfg(feature = "rkyv")]
impl<K: rkyv::Archive, V: rkyv::Archive> ExactSizeIterator for ArchivedKeys<'_, K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

#[cfg(feature = "rkyv")]
impl<K: rkyv::Archive, V: rkyv::Archive> core::iter::FusedIterator for ArchivedKeys<'_, K, V> {}

#[cfg(feature = "rkyv")]
impl<K: rkyv::Archive, V: rkyv::Archive> Clone for ArchivedKeys<'_, K, V> {
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}

#[cfg(feature = "rkyv")]
impl<K: rkyv::Archive, V: rkyv::Archive> fmt::Debug for ArchivedKeys<'_, K, V>
where
    K::Archived: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.iter.as_slice().iter().map(|entry| &entry.0))
            .finish()
    }
}

#[cfg(feature = "rkyv")]
impl<'a, K: rkyv::Archive, V: rkyv::Archive> Iterator for ArchivedValues<'a, K, V> {
    type Item = &'a V::Archived;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|entry| &entry.1)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth(n).map(|entry| &entry.1)
    }

    fn last(self) -> Option<Self::Item> {
        self.iter.last().map(|entry| &entry.1)
    }

    fn count(self) -> usize {
        self.iter.count()
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.fold(init, move |agg, entry| f(agg, &entry.1))
    }
}

#[cfg(feature = "rkyv")]
impl<K: rkyv::Archive, V: rkyv::Archive> DoubleEndedIterator for ArchivedValues<'_, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|entry| &entry.1)
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth_back(n).map(|entry| &entry.1)
    }

    fn rfold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.rfold(init, move |agg, entry| f(agg, &entry.1))
    }
}

#[cfg(feature = "rkyv")]
impl<K: rkyv::Archive, V: rkyv::Archive> ExactSizeIterator for ArchivedValues<'_, K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

#[cfg(feature = "rkyv")]
impl<K: rkyv::Archive, V: rkyv::Archive> core::iter::FusedIterator for ArchivedValues<'_, K, V> {}

#[cfg(feature = "rkyv")]
impl<K: rkyv::Archive, V: rkyv::Archive> Clone for ArchivedValues<'_, K, V> {
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}

#[cfg(feature = "rkyv")]
impl<K: rkyv::Archive, V: rkyv::Archive> fmt::Debug for ArchivedValues<'_, K, V>
where
    V::Archived: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.iter.as_slice().iter().map(|entry| &entry.1))
            .finish()
    }
}

#[cfg(feature = "rkyv")]
impl<'a, K: rkyv::Archive, V: rkyv::Archive> Iterator for ArchivedValuesSeal<'a, K, V> {
    type Item = rkyv::seal::Seal<'a, V::Archived>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|entry| rkyv::seal::Seal::new(&mut entry.1))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.iter
            .nth(n)
            .map(|entry| rkyv::seal::Seal::new(&mut entry.1))
    }

    fn last(self) -> Option<Self::Item> {
        self.iter
            .last()
            .map(|entry| rkyv::seal::Seal::new(&mut entry.1))
    }

    fn count(self) -> usize {
        self.iter.count()
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.fold(init, move |agg, entry| {
            f(agg, rkyv::seal::Seal::new(&mut entry.1))
        })
    }
}

#[cfg(feature = "rkyv")]
impl<K: rkyv::Archive, V: rkyv::Archive> DoubleEndedIterator for ArchivedValuesSeal<'_, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter
            .next_back()
            .map(|entry| rkyv::seal::Seal::new(&mut entry.1))
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter
            .nth_back(n)
            .map(|entry| rkyv::seal::Seal::new(&mut entry.1))
    }

    fn rfold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.rfold(init, move |agg, entry| {
            f(agg, rkyv::seal::Seal::new(&mut entry.1))
        })
    }
}

#[cfg(feature = "rkyv")]
impl<K: rkyv::Archive, V: rkyv::Archive> ExactSizeIterator for ArchivedValuesSeal<'_, K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

#[cfg(feature = "rkyv")]
impl<K: rkyv::Archive, V: rkyv::Archive> core::iter::FusedIterator
    for ArchivedValuesSeal<'_, K, V>
{
}

#[cfg(feature = "rkyv")]
impl<K: rkyv::Archive, V: rkyv::Archive> fmt::Debug for ArchivedValuesSeal<'_, K, V>
where
    V::Archived: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.iter.as_slice().iter().map(|entry| &entry.1))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::OrderedMap;

    use alloc::format;
    use alloc::vec::Vec;

    /// Every method below is overridden rather than taken from `Iterator`'s defaults, so each
    /// one has to be exercised to catch a delegation that reaches the wrong half of an entry.
    const ORDER: [(u32, u32); 5] = [(9, 90), (3, 30), (7, 70), (1, 10), (8, 80)];
    const KEYS: [u32; 5] = [9, 3, 7, 1, 8];
    const VALUES: [u32; 5] = [90, 30, 70, 10, 80];

    fn map() -> OrderedMap<u32, u32> {
        ORDER.into_iter().collect()
    }

    #[test]
    fn the_entry_iterators_delegate_every_override() {
        let map = map();

        assert_eq!(map.iter().size_hint(), (5, Some(5)));
        assert_eq!(map.iter().len(), 5);
        assert_eq!(map.iter().count(), 5);
        assert_eq!(map.iter().last(), Some((&8, &80)));
        assert_eq!(map.iter().nth(2), Some((&7, &70)));
        assert_eq!(map.iter().nth(9), None);
        assert_eq!(map.iter().fold(0u32, |acc, (k, _)| acc * 10 + k), 93718);
        assert_eq!(
            map.iter().rev().map(|(k, _)| *k).collect::<Vec<_>>(),
            [8, 1, 7, 3, 9]
        );
        assert_eq!(map.iter().nth_back(1), Some((&1, &10)));
        assert_eq!(map.iter().nth_back(9), None);
        assert_eq!(map.iter().rfold(0u32, |acc, (k, _)| acc * 10 + k), 81739);

        let mut iter = map.iter();
        iter.next();
        assert_eq!(iter.len(), 4);
        assert_eq!(
            iter.clone().map(|(k, _)| *k).collect::<Vec<_>>(),
            [3, 7, 1, 8]
        );
        assert!(format!("{iter:?}").contains("30"));

        assert_eq!(map.clone().into_iter().size_hint(), (5, Some(5)));
        assert_eq!(map.clone().into_iter().len(), 5);
        assert_eq!(map.clone().into_iter().count(), 5);
        assert_eq!(map.clone().into_iter().last(), Some((8, 80)));
        assert_eq!(map.clone().into_iter().nth(2), Some((7, 70)));
        assert_eq!(map.clone().into_iter().nth(9), None);
        assert_eq!(
            map.clone()
                .into_iter()
                .fold(0u32, |acc, (k, _)| acc * 10 + k),
            93718
        );
        assert_eq!(map.clone().into_iter().rev().collect::<Vec<_>>().len(), 5);
        assert_eq!(map.clone().into_iter().nth_back(1), Some((1, 10)));
        assert_eq!(map.clone().into_iter().nth_back(9), None);
        assert_eq!(
            map.clone()
                .into_iter()
                .rfold(0u32, |acc, (k, _)| acc * 10 + k),
            81739
        );

        let mut iter = map.into_iter();
        iter.next();
        assert_eq!(iter.len(), 4);
        assert_eq!(
            iter.clone().map(|(k, _)| k).collect::<Vec<_>>(),
            [3, 7, 1, 8]
        );
        assert!(format!("{iter:?}").contains("30"));
    }

    #[test]
    fn the_mutable_entry_iterator_delegates_every_override() {
        let mut map = map();

        assert_eq!(map.iter_mut().size_hint(), (5, Some(5)));
        assert_eq!(map.iter_mut().len(), 5);
        assert_eq!(map.iter_mut().count(), 5);
        assert_eq!(map.iter_mut().last().map(|(k, v)| (*k, *v)), Some((8, 80)));
        assert_eq!(map.iter_mut().nth(2).map(|(k, v)| (*k, *v)), Some((7, 70)));
        assert!(map.iter_mut().nth(9).is_none());
        assert_eq!(map.iter_mut().fold(0u32, |acc, (k, _)| acc * 10 + k), 93718);
        assert_eq!(
            map.iter_mut().rev().map(|(k, _)| *k).collect::<Vec<_>>(),
            [8, 1, 7, 3, 9]
        );
        assert_eq!(
            map.iter_mut().nth_back(1).map(|(k, v)| (*k, *v)),
            Some((1, 10))
        );
        assert!(map.iter_mut().nth_back(9).is_none());
        assert_eq!(
            map.iter_mut().rfold(0u32, |acc, (k, _)| acc * 10 + k),
            81739
        );

        let mut iter = map.iter_mut();
        iter.next();
        assert_eq!(iter.len(), 4);
        assert!(format!("{iter:?}").contains("30"));

        // The values are genuinely writable through it.
        for (_, value) in map.iter_mut() {
            *value += 1;
        }
        assert_eq!(map.get(&9u32), Some(&91));
    }

    #[test]
    fn the_key_iterators_yield_keys_and_the_value_iterators_yield_values() {
        let map = map();

        assert_eq!(map.keys().size_hint(), (5, Some(5)));
        assert_eq!(map.keys().len(), 5);
        assert_eq!(map.keys().count(), 5);
        assert_eq!(map.keys().last(), Some(&8));
        assert_eq!(map.keys().nth(2), Some(&7));
        assert_eq!(map.keys().nth(9), None);
        assert_eq!(map.keys().fold(0u32, |acc, k| acc * 10 + k), 93718);
        assert_eq!(map.keys().rev().collect::<Vec<_>>(), [&8, &1, &7, &3, &9]);
        assert_eq!(map.keys().nth_back(1), Some(&1));
        assert_eq!(map.keys().nth_back(9), None);
        assert_eq!(map.keys().rfold(0u32, |acc, k| acc * 10 + k), 81739);

        let mut keys = map.keys();
        keys.next();
        assert_eq!(keys.len(), 4);
        assert_eq!(keys.clone().collect::<Vec<_>>(), [&3, &7, &1, &8]);
        assert!(format!("{keys:?}").contains('3'));

        assert_eq!(map.values().size_hint(), (5, Some(5)));
        assert_eq!(map.values().len(), 5);
        assert_eq!(map.values().count(), 5);
        assert_eq!(map.values().last(), Some(&80));
        assert_eq!(map.values().nth(2), Some(&70));
        assert_eq!(map.values().nth(9), None);
        assert_eq!(map.values().fold(0u32, |acc, v| acc * 3 + v), 8840);
        assert_eq!(
            map.values().rev().collect::<Vec<_>>(),
            [&80, &10, &70, &30, &90]
        );
        assert_eq!(map.values().nth_back(1), Some(&10));
        assert_eq!(map.values().nth_back(9), None);
        assert_eq!(map.values().rfold(0u32, |acc, v| acc + v), 280);

        let mut values = map.values();
        values.next();
        assert_eq!(values.len(), 4);
        assert_eq!(values.clone().collect::<Vec<_>>(), [&30, &70, &10, &80]);
        assert!(format!("{values:?}").contains("30"));

        assert_eq!(map.clone().into_keys().collect::<Vec<_>>(), KEYS);
        assert_eq!(map.into_values().collect::<Vec<_>>(), VALUES);
    }

    #[test]
    fn the_owning_key_and_value_iterators_delegate_every_override() {
        assert_eq!(map().into_keys().size_hint(), (5, Some(5)));
        assert_eq!(map().into_keys().len(), 5);
        assert_eq!(map().into_keys().count(), 5);
        assert_eq!(map().into_keys().last(), Some(8));
        assert_eq!(map().into_keys().nth(2), Some(7));
        assert_eq!(map().into_keys().nth(9), None);
        assert_eq!(map().into_keys().fold(0u32, |acc, k| acc * 10 + k), 93718);
        assert_eq!(map().into_keys().rev().collect::<Vec<_>>(), [8, 1, 7, 3, 9]);
        assert_eq!(map().into_keys().nth_back(1), Some(1));
        assert_eq!(map().into_keys().nth_back(9), None);
        assert_eq!(map().into_keys().rfold(0u32, |acc, k| acc * 10 + k), 81739);

        let mut keys = map().into_keys();
        keys.next();
        assert_eq!(keys.len(), 4);
        assert_eq!(keys.clone().collect::<Vec<_>>(), [3, 7, 1, 8]);
        assert!(format!("{keys:?}").contains('3'));

        assert_eq!(map().into_values().size_hint(), (5, Some(5)));
        assert_eq!(map().into_values().len(), 5);
        assert_eq!(map().into_values().count(), 5);
        assert_eq!(map().into_values().last(), Some(80));
        assert_eq!(map().into_values().nth(2), Some(70));
        assert_eq!(map().into_values().nth(9), None);
        assert_eq!(map().into_values().fold(0u32, |acc, v| acc * 3 + v), 8840);
        assert_eq!(
            map().into_values().rev().collect::<Vec<_>>(),
            [80, 10, 70, 30, 90]
        );
        assert_eq!(map().into_values().nth_back(1), Some(10));
        assert_eq!(map().into_values().nth_back(9), None);
        assert_eq!(map().into_values().rfold(0u32, |acc, v| acc + v), 280);

        let mut values = map().into_values();
        values.next();
        assert_eq!(values.len(), 4);
        assert_eq!(values.clone().collect::<Vec<_>>(), [30, 70, 10, 80]);
        assert!(format!("{values:?}").contains("30"));
    }

    #[test]
    fn the_mutable_value_iterator_delegates_every_override() {
        let mut map = map();

        assert_eq!(map.values_mut().size_hint(), (5, Some(5)));
        assert_eq!(map.values_mut().len(), 5);
        assert_eq!(map.values_mut().count(), 5);
        assert_eq!(map.values_mut().last().copied(), Some(80));
        assert_eq!(map.values_mut().nth(2).copied(), Some(70));
        assert!(map.values_mut().nth(9).is_none());
        assert_eq!(map.values_mut().fold(0u32, |acc, v| acc + *v), 280);
        assert_eq!(
            map.values_mut().rev().map(|v| *v).collect::<Vec<_>>(),
            [80, 10, 70, 30, 90]
        );
        assert_eq!(map.values_mut().nth_back(1).copied(), Some(10));
        assert!(map.values_mut().nth_back(9).is_none());
        assert_eq!(map.values_mut().rfold(0u32, |acc, v| acc + *v), 280);

        let mut values = map.values_mut();
        values.next();
        assert_eq!(values.len(), 4);
        assert!(format!("{values:?}").contains("30"));

        for value in map.values_mut() {
            *value += 1;
        }
        assert_eq!(map.get(&9u32), Some(&91));
    }

    #[cfg(feature = "rkyv")]
    #[test]
    fn the_archived_iterators_delegate_every_override() {
        use crate::{PortableOrderedMap, rkyv::ArchivedOrderedMap};
        use rkyv::rancor::Error;
        use rkyv::seal::Seal;

        let map: PortableOrderedMap<u32, u32> = ORDER.into_iter().collect();
        let mut bytes = rkyv::to_bytes::<Error>(&map).unwrap();
        let archived = rkyv::access::<ArchivedOrderedMap<u32, u32>, Error>(&bytes).unwrap();

        let native = |v: &rkyv::rend::u32_le| v.to_native();

        assert_eq!(archived.iter().size_hint(), (5, Some(5)));
        assert_eq!(archived.iter().len(), 5);
        assert_eq!(archived.iter().count(), 5);
        assert_eq!(archived.iter().last().map(|(k, _)| native(k)), Some(8));
        assert_eq!(archived.iter().nth(2).map(|(k, _)| native(k)), Some(7));
        assert!(archived.iter().nth(9).is_none());
        assert_eq!(
            archived
                .iter()
                .fold(0u32, |acc, (k, _)| acc * 10 + native(k)),
            93718
        );
        assert_eq!(
            archived
                .iter()
                .rev()
                .map(|(k, _)| native(k))
                .collect::<Vec<_>>(),
            [8, 1, 7, 3, 9]
        );
        assert_eq!(archived.iter().nth_back(1).map(|(k, _)| native(k)), Some(1));
        assert!(archived.iter().nth_back(9).is_none());
        assert_eq!(
            archived
                .iter()
                .rfold(0u32, |acc, (k, _)| acc * 10 + native(k)),
            81739
        );

        let mut iter = archived.iter();
        iter.next();
        assert_eq!(iter.len(), 4);
        assert_eq!(
            iter.clone().map(|(k, _)| native(k)).collect::<Vec<_>>(),
            [3, 7, 1, 8]
        );
        assert!(format!("{iter:?}").contains("30"));

        assert_eq!(archived.keys().size_hint(), (5, Some(5)));
        assert_eq!(archived.keys().len(), 5);
        assert_eq!(archived.keys().count(), 5);
        assert_eq!(archived.keys().last().map(native), Some(8));
        assert_eq!(archived.keys().nth(2).map(native), Some(7));
        assert!(archived.keys().nth(9).is_none());
        assert_eq!(
            archived.keys().fold(0u32, |acc, k| acc * 10 + native(k)),
            93718
        );
        assert_eq!(
            archived.keys().rev().map(native).collect::<Vec<_>>(),
            [8, 1, 7, 3, 9]
        );
        assert_eq!(archived.keys().nth_back(1).map(native), Some(1));
        assert!(archived.keys().nth_back(9).is_none());
        assert_eq!(
            archived.keys().rfold(0u32, |acc, k| acc * 10 + native(k)),
            81739
        );
        let mut keys = archived.keys();
        keys.next();
        assert_eq!(keys.clone().map(native).collect::<Vec<_>>(), [3, 7, 1, 8]);
        assert!(format!("{keys:?}").contains('3'));

        assert_eq!(archived.values().size_hint(), (5, Some(5)));
        assert_eq!(archived.values().len(), 5);
        assert_eq!(archived.values().count(), 5);
        assert_eq!(archived.values().last().map(native), Some(80));
        assert_eq!(archived.values().nth(2).map(native), Some(70));
        assert!(archived.values().nth(9).is_none());
        assert_eq!(archived.values().fold(0u32, |acc, v| acc + native(v)), 280);
        assert_eq!(
            archived.values().rev().map(native).collect::<Vec<_>>(),
            [80, 10, 70, 30, 90]
        );
        assert_eq!(archived.values().nth_back(1).map(native), Some(10));
        assert!(archived.values().nth_back(9).is_none());
        assert_eq!(archived.values().rfold(0u32, |acc, v| acc + native(v)), 280);
        let mut values = archived.values();
        values.next();
        assert_eq!(values.len(), 4);
        assert_eq!(
            values.clone().map(native).collect::<Vec<_>>(),
            [30, 70, 10, 80]
        );
        assert!(format!("{values:?}").contains("30"));

        // The sealed iterators reach the same entries, with the values writable.
        type Archived = ArchivedOrderedMap<u32, u32>;
        {
            let a = rkyv::access_mut::<Archived, Error>(&mut bytes).unwrap();
            let mut iter = Archived::iter_seal(a);
            assert_eq!(iter.size_hint(), (5, Some(5)));
            assert_eq!(iter.len(), 5);
            assert_eq!(iter.next().map(|(k, _)| native(k)), Some(9));
            assert_eq!(iter.nth(1).map(|(k, _)| native(k)), Some(7));
            assert_eq!(iter.next_back().map(|(k, _)| native(k)), Some(8));
            assert!(format!("{iter:?}").contains("10"));
            assert_eq!(iter.count(), 1);
        }
        {
            let a = rkyv::access_mut::<Archived, Error>(&mut bytes).unwrap();
            assert_eq!(
                Archived::iter_seal(a).nth_back(1).map(|(k, _)| native(k)),
                Some(1)
            );
        }
        {
            let a = rkyv::access_mut::<Archived, Error>(&mut bytes).unwrap();
            assert_eq!(
                Archived::iter_seal(a).rfold(0u32, |acc, (k, _)| acc * 10 + native(k)),
                81739,
            );
        }
        {
            let a = rkyv::access_mut::<Archived, Error>(&mut bytes).unwrap();
            assert_eq!(
                Archived::iter_seal(a).fold(0u32, |acc, (k, _)| acc * 10 + native(k)),
                93718,
            );
        }
        {
            let a = rkyv::access_mut::<Archived, Error>(&mut bytes).unwrap();
            assert_eq!(
                Archived::iter_seal(a).last().map(|(k, _)| native(k)),
                Some(8)
            );
        }
        {
            let a = rkyv::access_mut::<Archived, Error>(&mut bytes).unwrap();
            let mut values = Archived::values_seal(a);
            assert_eq!(values.size_hint(), (5, Some(5)));
            assert_eq!(values.len(), 5);
            assert_eq!(values.next().map(|v| *Seal::unseal(v)), Some(90.into()));
            assert_eq!(values.nth(1).map(|v| *Seal::unseal(v)), Some(70.into()));
            assert_eq!(
                values.next_back().map(|v| *Seal::unseal(v)),
                Some(80.into())
            );
            assert!(format!("{values:?}").contains("10"));
            assert_eq!(values.count(), 1);
        }
        {
            let a = rkyv::access_mut::<Archived, Error>(&mut bytes).unwrap();
            assert_eq!(
                Archived::values_seal(a)
                    .nth_back(1)
                    .map(|v| *Seal::unseal(v)),
                Some(10.into()),
            );
        }
        {
            let a = rkyv::access_mut::<Archived, Error>(&mut bytes).unwrap();
            assert_eq!(
                Archived::values_seal(a).rfold(0u32, |acc, v| acc + Seal::unseal(v).to_native()),
                280,
            );
        }
        {
            let a = rkyv::access_mut::<Archived, Error>(&mut bytes).unwrap();
            assert_eq!(
                Archived::values_seal(a).fold(0u32, |acc, v| acc + Seal::unseal(v).to_native()),
                280,
            );
        }
        {
            let a = rkyv::access_mut::<Archived, Error>(&mut bytes).unwrap();
            assert_eq!(
                Archived::values_seal(a).last().map(|v| *Seal::unseal(v)),
                Some(80.into()),
            );
        }
    }
}
