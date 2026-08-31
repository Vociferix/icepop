use core::fmt;

pub struct Iter<'a, K, V> {
    pub(crate) iter: core::slice::Iter<'a, (K, V)>,
}

pub struct IterMut<'a, K, V> {
    pub(crate) iter: core::slice::IterMut<'a, (K, V)>,
}

#[derive(Clone)]
pub struct IntoIter<K, V> {
    pub(crate) iter: alloc::vec::IntoIter<(K, V)>,
}

pub struct Keys<'a, K, V> {
    pub(crate) iter: core::slice::Iter<'a, (K, V)>,
}

#[derive(Clone)]
pub struct IntoKeys<K, V> {
    pub(crate) iter: alloc::vec::IntoIter<(K, V)>,
}

pub struct Values<'a, K, V> {
    pub(crate) iter: core::slice::Iter<'a, (K, V)>,
}

pub struct ValuesMut<'a, K, V> {
    pub(crate) iter: core::slice::IterMut<'a, (K, V)>,
}

#[derive(Clone)]
pub struct IntoValues<K, V> {
    pub(crate) iter: alloc::vec::IntoIter<(K, V)>,
}

#[cfg(feature = "rkyv")]
pub struct ArchivedIter<'a, K: rkyv::Archive, V: rkyv::Archive> {
    pub(crate) iter: core::slice::Iter<'a, rkyv::tuple::ArchivedTuple2<K::Archived, V::Archived>>,
}

#[cfg(feature = "rkyv")]
pub struct ArchivedIterSeal<'a, K: rkyv::Archive, V: rkyv::Archive> {
    pub(crate) iter:
        core::slice::IterMut<'a, rkyv::tuple::ArchivedTuple2<K::Archived, V::Archived>>,
}

#[cfg(feature = "rkyv")]
pub struct ArchivedKeys<'a, K: rkyv::Archive, V: rkyv::Archive> {
    pub(crate) iter: core::slice::Iter<'a, rkyv::tuple::ArchivedTuple2<K::Archived, V::Archived>>,
}

#[cfg(feature = "rkyv")]
pub struct ArchivedValues<'a, K: rkyv::Archive, V: rkyv::Archive> {
    pub(crate) iter: core::slice::Iter<'a, rkyv::tuple::ArchivedTuple2<K::Archived, V::Archived>>,
}

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
impl<'a, K: rkyv::Archive, V: rkyv::Archive> fmt::Debug for ArchivedIter<'_, K, V>
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
impl<'a, K: rkyv::Archive, V: rkyv::Archive> fmt::Debug for ArchivedIterSeal<'_, K, V>
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
impl<'a, K: rkyv::Archive, V: rkyv::Archive> fmt::Debug for ArchivedKeys<'_, K, V>
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
impl<'a, K: rkyv::Archive, V: rkyv::Archive> fmt::Debug for ArchivedValues<'_, K, V>
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
impl<'a, K: rkyv::Archive, V: rkyv::Archive> fmt::Debug for ArchivedValuesSeal<'_, K, V>
where
    V::Archived: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.iter.as_slice().iter().map(|entry| &entry.1))
            .finish()
    }
}
