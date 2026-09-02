//! Iterators shared by [`Set`](crate::Set) and [`OrderedSet`](crate::OrderedSet).

use core::fmt;

/// Borrowing iterator over a set's elements.
///
/// Created by [`Set::iter`](crate::Set::iter), [`OrderedSet::iter`](crate::OrderedSet::iter)
/// and the corresponding builders. Elements arrive in the collection's own order: insertion
/// order for an ordered set, an arbitrary order otherwise.
///
/// # Example
///
/// ```
/// use icepop_phf::OrderedSet;
///
/// let set: OrderedSet<u32> = [3u32, 1, 2].into_iter().collect();
///
/// assert_eq!(set.iter().copied().collect::<Vec<_>>(), [3, 1, 2]);
/// ```
pub struct Iter<'a, T> {
    pub(crate) iter: core::slice::Iter<'a, T>,
}

/// Owning iterator over a set's elements.
///
/// Created by [`IntoIterator`] on a set or its builder.
///
/// # Example
///
/// ```
/// use icepop_phf::OrderedSet;
///
/// let set: OrderedSet<String> = ["ash".to_string()].into_iter().collect();
///
/// assert_eq!(set.into_iter().collect::<Vec<_>>(), ["ash".to_string()]);
/// ```
#[derive(Clone)]
pub struct IntoIter<T> {
    pub(crate) iter: alloc::vec::IntoIter<T>,
}

/// Borrowing iterator over an archived set's elements.
///
/// Created by [`ArchivedSet::iter`](crate::rkyv::ArchivedSet::iter) and
/// [`ArchivedOrderedSet::iter`](crate::rkyv::ArchivedOrderedSet::iter). Yields elements in
/// their archived form.
///
/// # Example
///
/// ```
/// use icepop_phf::{PortableOrderedSet, rkyv::ArchivedOrderedSet};
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
#[cfg(feature = "rkyv")]
pub struct ArchivedIter<'a, T: rkyv::Archive> {
    pub(crate) iter: core::slice::Iter<'a, T::Archived>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

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

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
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

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, T> core::iter::FusedIterator for Iter<'a, T> {}

impl<'a, T> Clone for Iter<'a, T> {
    fn clone(&self) -> Self {
        Iter {
            iter: self.iter.clone(),
        }
    }
}

impl<'a, T> fmt::Debug for Iter<'a, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter.as_slice()).finish()
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

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

impl<T> DoubleEndedIterator for IntoIter<T> {
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

impl<T> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<T> core::iter::FusedIterator for IntoIter<T> {}

impl<T> fmt::Debug for IntoIter<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter.as_slice()).finish()
    }
}

#[cfg(feature = "rkyv")]
impl<'a, T: rkyv::Archive> Iterator for ArchivedIter<'a, T> {
    type Item = &'a T::Archived;

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

#[cfg(feature = "rkyv")]
impl<T: rkyv::Archive> DoubleEndedIterator for ArchivedIter<'_, T> {
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

#[cfg(feature = "rkyv")]
impl<T: rkyv::Archive> ExactSizeIterator for ArchivedIter<'_, T> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

#[cfg(feature = "rkyv")]
impl<T: rkyv::Archive> core::iter::FusedIterator for ArchivedIter<'_, T> {}

#[cfg(feature = "rkyv")]
impl<T: rkyv::Archive> Clone for ArchivedIter<'_, T> {
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}

#[cfg(feature = "rkyv")]
impl<T: rkyv::Archive> fmt::Debug for ArchivedIter<'_, T>
where
    T::Archived: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter.as_slice()).finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::OrderedSet;

    use alloc::format;
    use alloc::vec::Vec;

    /// Every method below is overridden rather than taken from `Iterator`'s defaults, so each
    /// one has to be exercised to catch a delegation that reaches the wrong element.
    const ORDER: [u32; 5] = [9, 3, 7, 1, 8];

    fn set() -> OrderedSet<u32> {
        ORDER.into_iter().collect()
    }

    #[test]
    fn the_borrowing_iterator_delegates_every_override() {
        let set = set();

        assert_eq!(set.iter().size_hint(), (5, Some(5)));
        assert_eq!(set.iter().len(), 5);
        assert_eq!(set.iter().count(), 5);
        assert_eq!(set.iter().last(), Some(&8));
        assert_eq!(set.iter().nth(2), Some(&7));
        assert_eq!(set.iter().nth(9), None);
        assert_eq!(set.iter().fold(0u32, |acc, v| acc * 10 + v), 93718);

        assert_eq!(set.iter().rev().collect::<Vec<_>>(), [&8, &1, &7, &3, &9]);
        assert_eq!(set.iter().nth_back(1), Some(&1));
        assert_eq!(set.iter().nth_back(9), None);
        assert_eq!(set.iter().rfold(0u32, |acc, v| acc * 10 + v), 81739);

        // A partly consumed iterator reports what is left, and its clone resumes from there.
        let mut iter = set.iter();
        iter.next();
        assert_eq!(iter.len(), 4);
        assert_eq!(iter.clone().collect::<Vec<_>>(), [&3, &7, &1, &8]);
        assert!(format!("{iter:?}").contains('3'));
    }

    #[test]
    fn the_owning_iterator_delegates_every_override() {
        assert_eq!(set().into_iter().size_hint(), (5, Some(5)));
        assert_eq!(set().into_iter().len(), 5);
        assert_eq!(set().into_iter().count(), 5);
        assert_eq!(set().into_iter().last(), Some(8));
        assert_eq!(set().into_iter().nth(2), Some(7));
        assert_eq!(set().into_iter().nth(9), None);
        assert_eq!(set().into_iter().fold(0u32, |acc, v| acc * 10 + v), 93718);

        assert_eq!(set().into_iter().rev().collect::<Vec<_>>(), [8, 1, 7, 3, 9]);
        assert_eq!(set().into_iter().nth_back(1), Some(1));
        assert_eq!(set().into_iter().nth_back(9), None);
        assert_eq!(set().into_iter().rfold(0u32, |acc, v| acc * 10 + v), 81739);

        let mut iter = set().into_iter();
        iter.next();
        assert_eq!(iter.len(), 4);
        assert_eq!(iter.clone().collect::<Vec<_>>(), [3, 7, 1, 8]);
        assert!(format!("{iter:?}").contains('3'));
    }

    #[cfg(feature = "rkyv")]
    #[test]
    fn the_archived_iterator_delegates_every_override() {
        use crate::{PortableOrderedSet, rkyv::ArchivedOrderedSet};
        use rkyv::rancor::Error;

        let set: PortableOrderedSet<u32> = ORDER.into_iter().collect();
        let bytes = rkyv::to_bytes::<Error>(&set).unwrap();
        let archived = rkyv::access::<ArchivedOrderedSet<u32>, Error>(&bytes).unwrap();

        let native = |v: &rkyv::rend::u32_le| v.to_native();

        assert_eq!(archived.iter().size_hint(), (5, Some(5)));
        assert_eq!(archived.iter().len(), 5);
        assert_eq!(archived.iter().count(), 5);
        assert_eq!(archived.iter().last().map(native), Some(8));
        assert_eq!(archived.iter().nth(2).map(native), Some(7));
        assert_eq!(archived.iter().nth(9).map(native), None);
        assert_eq!(
            archived
                .iter()
                .fold(0u32, |acc, v| acc * 10 + v.to_native()),
            93718
        );

        assert_eq!(
            archived.iter().rev().map(native).collect::<Vec<_>>(),
            [8, 1, 7, 3, 9]
        );
        assert_eq!(archived.iter().nth_back(1).map(native), Some(1));
        assert_eq!(archived.iter().nth_back(9).map(native), None);
        assert_eq!(
            archived
                .iter()
                .rfold(0u32, |acc, v| acc * 10 + v.to_native()),
            81739
        );

        let mut iter = archived.iter();
        iter.next();
        assert_eq!(iter.len(), 4);
        assert_eq!(iter.clone().map(native).collect::<Vec<_>>(), [3, 7, 1, 8]);
        assert!(format!("{iter:?}").contains('3'));
    }
}
