//! A minimal perfect hash set whose entries are stored in an arbitrary order.

use crate::portability::{NonPortable, Portable, SetOps};
use crate::table::{Builder as TableBuilder, Table};

use ::portable::DefaultHasherSeed;

#[doc(inline)]
pub use crate::set_iters::{IntoIter, Iter};

#[cfg(feature = "rkyv")]
pub mod rkyv;

mod non_portable;
mod portable;

/// A minimal perfect hash set: built once, then read-only.
///
/// Membership tests examine exactly one entry and the table stores no empty slots, but the
/// whole element set must be known up front and the result cannot be modified. Build one by
/// filling a [`Builder`] and calling `build`.
///
/// Entries are permuted at build time so that an element's hash slot is its index, which is
/// what lets a lookup avoid an indirection. Iteration and [`as_slice`](Self::as_slice)
/// therefore visit an arbitrary order; use [`OrderedSet`](crate::OrderedSet) to keep the order
/// elements were inserted.
///
/// `P` selects the lookup interface and is either [`NonPortable`], the default, or
/// [`Portable`]; see the [crate documentation](crate#portability). [`PortableSet`] names the
/// latter.
///
/// # Example
///
/// ```
/// use icepop_phf::Set;
///
/// let mut builder = Set::<&str>::builder();
/// builder.insert("ash");
/// builder.insert("elm");
///
/// let set = builder.build();
///
/// assert!(set.contains("elm"));
/// assert!(!set.contains("oak"));
/// assert_eq!(set.len(), 2);
/// ```
pub struct Set<T, S = DefaultHasherSeed, P = NonPortable> {
    table: Table<SetOps<T>, S, P>,
}

/// Accumulates the elements of a [`Set`], then freezes them into one.
///
/// An ordinary mutable hash set until [`build`](Self::build) is called. Building is where the
/// minimal perfect hash function is constructed, so it is much more expensive than an insert
/// and should happen once, after all elements are known.
///
/// # Example
///
/// ```
/// use icepop_phf::set::Builder;
///
/// let mut builder = Builder::<u32>::new();
/// builder.insert(7);
/// builder.insert(11);
/// assert!(!builder.insert(7));
///
/// let set = builder.build();
/// assert_eq!(set.len(), 2);
/// ```
pub struct Builder<T, S = DefaultHasherSeed, P = NonPortable> {
    builder: TableBuilder<SetOps<T>, S, P>,
}

/// A [`Set`] that hashes and compares identically on every platform.
///
/// The form that supports `serde` and `rkyv`. See [`Portable`].
///
/// # Example
///
/// ```
/// use icepop_phf::PortableSet;
///
/// let set: PortableSet<u32> = [1u32, 2, 3].into_iter().collect();
///
/// assert!(set.contains(&2u32));
/// ```
pub type PortableSet<T, S = DefaultHasherSeed> = Set<T, S, Portable>;

/// The [`Builder`] that produces a [`PortableSet`].
///
/// # Example
///
/// ```
/// use icepop_phf::set::PortableBuilder;
///
/// let mut builder = PortableBuilder::<u32>::new();
/// builder.insert(42);
///
/// assert!(builder.build().contains(&42u32));
/// ```
pub type PortableBuilder<T, S = DefaultHasherSeed> = Builder<T, S, Portable>;

impl<T, S, P> Set<T, S, P> {
    /// Returns the hasher the set was built with.
    ///
    /// Lookups reuse it, so it is kept for the life of the set and travels with it through
    /// serialization.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{DefaultHasherSeed, Set};
    ///
    /// let set = Set::<u32>::builder_with_hasher(DefaultHasherSeed::with_seed(42)).build();
    ///
    /// assert_eq!(set.hasher().seed(), 42);
    /// ```
    pub fn hasher(&self) -> &S {
        self.table.hasher()
    }

    /// Returns `true` if the set contains no elements.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Set;
    ///
    /// assert!(Set::<u32>::builder().build().is_empty());
    /// assert!(![1u32].into_iter().collect::<Set<u32>>().is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// Returns the number of elements in the set.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Set;
    ///
    /// let set: Set<u32> = [1u32, 2, 3].into_iter().collect();
    ///
    /// assert_eq!(set.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Returns an iterator over the elements, in an arbitrary order.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Set;
    ///
    /// let set: Set<u32> = [1u32, 2, 3].into_iter().collect();
    ///
    /// let mut elements: Vec<_> = set.iter().copied().collect();
    /// elements.sort();
    /// assert_eq!(elements, [1, 2, 3]);
    /// ```
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            iter: self.table.iter(),
        }
    }

    /// Borrows the elements as a contiguous slice, in an arbitrary order.
    ///
    /// The order is fixed once the set is built, so an index obtained from
    /// [`get_index`](Self::get_index) addresses this slice.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Set;
    ///
    /// let set: Set<u32> = [1u32, 2, 3].into_iter().collect();
    ///
    /// let mut elements = set.as_slice().to_vec();
    /// elements.sort();
    /// assert_eq!(elements, [1, 2, 3]);
    /// ```
    pub fn as_slice(&self) -> &[T] {
        self.table.as_slice()
    }

    /// Returns the element at `index`, or `None` if it is out of bounds.
    ///
    /// Indices run over an arbitrary order, so this is only meaningful with an index from
    /// [`get_index`](Self::get_index) or from enumerating [`as_slice`](Self::as_slice).
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Set;
    ///
    /// let set: Set<u32> = [10u32, 20].into_iter().collect();
    /// let index = set.get_index(&20u32).unwrap();
    ///
    /// assert_eq!(set.index(index), Some(&20));
    /// assert_eq!(set.index(2), None);
    /// ```
    pub fn index(&self, index: usize) -> Option<&T> {
        self.table.index(index)
    }

    /// Returns the element at `index` without a bounds check.
    ///
    /// # Safety
    ///
    /// `index` must be less than [`len`](Self::len).
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Set;
    ///
    /// let set: Set<u32> = [10u32, 20].into_iter().collect();
    /// let index = set.get_index(&20u32).unwrap();
    ///
    /// // SAFETY: `get_index` returned this index, so it is in bounds.
    /// assert_eq!(unsafe { set.index_unchecked(index) }, &20);
    /// ```
    pub unsafe fn index_unchecked(&self, index: usize) -> &T {
        // SAFETY: `Table::index_unchecked` has this function's contract, which the caller upheld.
        unsafe { self.table.index_unchecked(index) }
    }
}

impl<T, S, P> core::fmt::Debug for Set<T, S, P>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl<T, S, P> Clone for Set<T, S, P>
where
    T: Clone,
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            table: self.table.clone(),
        }
    }

    fn clone_from(&mut self, other: &Self) {
        self.table.clone_from(&other.table);
    }
}

impl<'a, T, S, P> IntoIterator for &'a Set<T, S, P> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T, S, P> IntoIterator for Set<T, S, P> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            iter: self.table.into_iter(),
        }
    }
}

impl<T, S, P> Builder<T, S, P> {
    /// Returns the hasher the builder will hand to the set it builds.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{DefaultHasherSeed, Set};
    ///
    /// let builder = Set::<u32>::builder_with_hasher(DefaultHasherSeed::with_seed(42));
    ///
    /// assert_eq!(builder.hasher().seed(), 42);
    /// ```
    pub fn hasher(&self) -> &S {
        self.builder.hasher()
    }

    /// Returns `true` if no elements have been inserted.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Set;
    ///
    /// let mut builder = Set::<u32>::builder();
    /// assert!(builder.is_empty());
    ///
    /// builder.insert(1);
    /// assert!(!builder.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.builder.is_empty()
    }

    /// Returns the number of elements inserted so far.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Set;
    ///
    /// let mut builder = Set::<u32>::builder();
    /// builder.insert(1);
    /// builder.insert(2);
    ///
    /// assert_eq!(builder.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.builder.len()
    }

    /// Returns how many elements the builder can hold before it reallocates.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Set;
    ///
    /// let builder = Set::<u32>::builder_with_capacity(10);
    ///
    /// assert!(builder.capacity() >= 10);
    /// ```
    pub fn capacity(&self) -> usize {
        self.builder.capacity()
    }

    /// Returns an iterator over the elements inserted so far, in an arbitrary order.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Set;
    ///
    /// let mut builder = Set::<u32>::builder();
    /// builder.insert(1);
    /// builder.insert(2);
    ///
    /// let mut elements: Vec<_> = builder.iter().copied().collect();
    /// elements.sort();
    /// assert_eq!(elements, [1, 2]);
    /// ```
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            iter: self.builder.iter(),
        }
    }

    /// Reserves room for at least `additional` more elements.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Set;
    ///
    /// let mut builder = Set::<u32>::builder();
    /// builder.reserve(100);
    ///
    /// assert!(builder.capacity() >= 100);
    /// ```
    pub fn reserve(&mut self, additional: usize) {
        self.builder.reserve(additional);
    }

    /// Drops capacity beyond what the inserted elements need.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Set;
    ///
    /// let mut builder = Set::<u32>::builder_with_capacity(100);
    /// builder.insert(1);
    /// builder.shrink_to_fit();
    ///
    /// assert!(builder.capacity() < 100);
    /// ```
    pub fn shrink_to_fit(&mut self) {
        self.builder.shrink_to_fit();
    }

    /// Drops capacity down towards `min_capacity`, never below the current length.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Set;
    ///
    /// let mut builder = Set::<u32>::builder_with_capacity(100);
    /// builder.insert(1);
    /// builder.shrink_to(10);
    ///
    /// assert!(builder.capacity() < 100);
    /// ```
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.builder.shrink_to(min_capacity);
    }

    /// Removes every element, keeping the allocated capacity.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::Set;
    ///
    /// let mut builder = Set::<u32>::builder();
    /// builder.insert(1);
    /// builder.clear();
    ///
    /// assert!(builder.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.builder.clear();
    }
}

impl<T, S, P> core::fmt::Debug for Builder<T, S, P>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl<T, S, P> Clone for Builder<T, S, P>
where
    T: Clone,
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            builder: self.builder.clone(),
        }
    }

    fn clone_from(&mut self, other: &Self) {
        self.builder.clone_from(&other.builder);
    }
}

impl<'a, T, S, P> IntoIterator for &'a Builder<T, S, P> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T, S, P> IntoIterator for Builder<T, S, P> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            iter: self.builder.into_iter(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::format;
    use alloc::vec::Vec;

    fn set_of(keys: impl IntoIterator<Item = u32>) -> Set<u32> {
        let mut builder = Set::<u32>::builder_with_hasher(DefaultHasherSeed::with_seed(1));
        for k in keys {
            builder.insert(k);
        }
        builder.build()
    }

    #[test]
    fn a_built_set_reports_its_contents() {
        let set = set_of(0..5);

        assert_eq!(set.len(), 5);
        assert!(!set.is_empty());
        assert_eq!(set.hasher().seed(), 1);
        assert_eq!(set.as_slice().len(), 5);

        let mut seen = set.iter().copied().collect::<Vec<_>>();
        seen.sort_unstable();
        assert_eq!(seen, [0, 1, 2, 3, 4]);

        let index = set.get_index(&3u32).unwrap();
        assert_eq!(set.index(index), Some(&3));
        assert_eq!(set.index(5), None);
        // SAFETY: `get_index` returned this index, so it is in bounds.
        assert_eq!(unsafe { set.index_unchecked(index) }, &3);

        assert!(Set::<u32>::builder().build().is_empty());
    }

    #[test]
    fn a_builder_reports_its_contents_and_capacity() {
        let mut builder =
            Set::<u32>::builder_with_capacity_and_hasher(50, DefaultHasherSeed::with_seed(2));
        assert!(builder.is_empty());
        assert!(builder.capacity() >= 50);
        assert_eq!(builder.hasher().seed(), 2);

        builder.insert(1);
        builder.insert(2);
        assert_eq!(builder.len(), 2);
        assert_eq!(builder.iter().count(), 2);

        builder.shrink_to(4);
        assert!(builder.capacity() < 50);
        builder.reserve(100);
        assert!(builder.capacity() >= 100);
        builder.shrink_to_fit();
        assert!(builder.capacity() < 100);

        builder.clear();
        assert!(builder.is_empty());
    }

    #[test]
    fn both_forms_iterate_by_reference_and_by_value() {
        let set = set_of(0..3);

        let mut borrowed = (&set).into_iter().copied().collect::<Vec<_>>();
        borrowed.sort_unstable();
        assert_eq!(borrowed, [0, 1, 2]);

        let mut builder = Set::<u32>::builder();
        builder.insert(7);
        assert_eq!((&builder).into_iter().collect::<Vec<_>>(), [&7]);
        assert_eq!(builder.into_iter().collect::<Vec<_>>(), [7]);

        let mut owned = set.into_iter().collect::<Vec<_>>();
        owned.sort_unstable();
        assert_eq!(owned, [0, 1, 2]);
    }

    #[test]
    fn cloning_and_formatting_reach_every_element() {
        let set = set_of(0..3);

        let mut clone = set.clone();
        assert_eq!(clone.len(), 3);
        clone.clone_from(&set);
        assert!(clone.contains(&1u32));

        // The order is arbitrary, so only the bracketing and the membership are pinned.
        let shown = format!("{set:?}");
        assert!(shown.starts_with('{') && shown.ends_with('}'), "{shown}");
        for k in 0..3 {
            assert!(shown.contains(&format!("{k}")), "{shown}");
        }

        let mut builder = Set::<u32>::builder();
        builder.insert(1);
        let mut builder_clone = builder.clone();
        builder_clone.insert(2);
        builder.clone_from(&builder_clone);
        assert_eq!(builder.len(), 2);
        assert!(format!("{builder:?}").contains('1'));
    }
}
