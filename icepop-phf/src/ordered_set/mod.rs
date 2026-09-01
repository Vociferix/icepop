//! A minimal perfect hash set that keeps the order its elements were inserted.

use crate::portability::{NonPortable, OrderedSetOps, Portable};
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
/// Entries stay in the order they were inserted into the builder, so iteration,
/// [`as_slice`](Self::as_slice) and [`index`](Self::index) all follow that order. Paying for it
/// costs four bytes per entry for a slot-to-entry table, and one extra indirection per lookup.
/// Use [`Set`](crate::Set) when the order does not matter.
///
/// `P` selects the lookup interface and is either [`NonPortable`], the default, or
/// [`Portable`]; see the [crate documentation](crate#portability). [`PortableOrderedSet`] names the
/// latter.
///
/// # Example
///
/// ```
/// use icepop_phf::OrderedSet;
///
/// let mut builder = OrderedSet::<&str>::builder();
/// builder.insert("ash");
/// builder.insert("elm");
///
/// let set = builder.build();
///
/// assert!(set.contains("elm"));
/// assert!(!set.contains("oak"));
/// assert_eq!(set.len(), 2);
/// ```
pub struct OrderedSet<T, S = DefaultHasherSeed, P = NonPortable> {
    table: Table<OrderedSetOps<T>, S, P>,
}

/// Accumulates the elements of a [`OrderedSet`], then freezes them into one.
///
/// An ordinary mutable hash set until [`build`](Self::build) is called. Building is where the
/// minimal perfect hash function is constructed, so it is much more expensive than an insert
/// and should happen once, after all elements are known.
///
/// # Example
///
/// ```
/// use icepop_phf::ordered_set::Builder;
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
    builder: TableBuilder<OrderedSetOps<T>, S, P>,
}

/// A [`OrderedSet`] that hashes and compares identically on every platform.
///
/// The form that supports `serde` and `rkyv`. See [`Portable`].
///
/// # Example
///
/// ```
/// use icepop_phf::PortableOrderedSet;
///
/// let set: PortableOrderedSet<u32> = [1u32, 2, 3].into_iter().collect();
///
/// assert!(set.contains(&2u32));
/// ```
pub type PortableOrderedSet<T, S = DefaultHasherSeed> = OrderedSet<T, S, Portable>;

/// The [`Builder`] that produces a [`PortableOrderedSet`].
///
/// # Example
///
/// ```
/// use icepop_phf::ordered_set::PortableBuilder;
///
/// let mut builder = PortableBuilder::<u32>::new();
/// builder.insert(42);
///
/// assert!(builder.build().contains(&42u32));
/// ```
pub type PortableBuilder<T, S = DefaultHasherSeed> = Builder<T, S, Portable>;

impl<T, S, P> OrderedSet<T, S, P> {
    /// Returns the hasher the set was built with.
    ///
    /// Lookups reuse it, so it is kept for the life of the set and travels with it through
    /// serialization.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::{DefaultHasherSeed, OrderedSet};
    ///
    /// let set = OrderedSet::<u32>::builder_with_hasher(DefaultHasherSeed::with_seed(42)).build();
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
    /// use icepop_phf::OrderedSet;
    ///
    /// assert!(OrderedSet::<u32>::builder().build().is_empty());
    /// assert!(![1u32].into_iter().collect::<OrderedSet<u32>>().is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// Returns the number of elements in the set.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::OrderedSet;
    ///
    /// let set: OrderedSet<u32> = [1u32, 2, 3].into_iter().collect();
    ///
    /// assert_eq!(set.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Returns an iterator over the elements, in insertion order.
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
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            iter: self.table.iter(),
        }
    }

    /// Borrows the elements as a contiguous slice, in insertion order.
    ///
    /// An index obtained from [`get_index`](Self::get_index) addresses this slice.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::OrderedSet;
    ///
    /// let set: OrderedSet<u32> = [3u32, 1, 2].into_iter().collect();
    ///
    /// assert_eq!(set.as_slice(), &[3, 1, 2]);
    /// ```
    pub fn as_slice(&self) -> &[T] {
        self.table.as_slice()
    }

    /// Returns the element at `index`, or `None` if it is out of bounds.
    ///
    /// Indices run over insertion order.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::OrderedSet;
    ///
    /// let set: OrderedSet<u32> = [10u32, 20].into_iter().collect();
    ///
    /// assert_eq!(set.index(0), Some(&10));
    /// assert_eq!(set.index(1), Some(&20));
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
    /// use icepop_phf::OrderedSet;
    ///
    /// let set: OrderedSet<u32> = [10u32, 20].into_iter().collect();
    ///
    /// // SAFETY: the set has two elements, so index 1 is in bounds.
    /// assert_eq!(unsafe { set.index_unchecked(1) }, &20);
    /// ```
    pub unsafe fn index_unchecked(&self, index: usize) -> &T {
        unsafe { self.table.index_unchecked(index) }
    }
}

impl<T, S, P> core::fmt::Debug for OrderedSet<T, S, P>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl<T, S, P> Clone for OrderedSet<T, S, P>
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

impl<'a, T, S, P> IntoIterator for &'a OrderedSet<T, S, P> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T, S, P> IntoIterator for OrderedSet<T, S, P> {
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
    /// use icepop_phf::{DefaultHasherSeed, OrderedSet};
    ///
    /// let builder = OrderedSet::<u32>::builder_with_hasher(DefaultHasherSeed::with_seed(42));
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
    /// use icepop_phf::OrderedSet;
    ///
    /// let mut builder = OrderedSet::<u32>::builder();
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
    /// use icepop_phf::OrderedSet;
    ///
    /// let mut builder = OrderedSet::<u32>::builder();
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
    /// use icepop_phf::OrderedSet;
    ///
    /// let builder = OrderedSet::<u32>::builder_with_capacity(10);
    ///
    /// assert!(builder.capacity() >= 10);
    /// ```
    pub fn capacity(&self) -> usize {
        self.builder.capacity()
    }

    /// Returns an iterator over the elements inserted so far, in insertion order.
    ///
    /// # Example
    ///
    /// ```
    /// use icepop_phf::OrderedSet;
    ///
    /// let mut builder = OrderedSet::<u32>::builder();
    /// builder.insert(2);
    /// builder.insert(1);
    ///
    /// assert_eq!(builder.iter().copied().collect::<Vec<_>>(), [2, 1]);
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
    /// use icepop_phf::OrderedSet;
    ///
    /// let mut builder = OrderedSet::<u32>::builder();
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
    /// use icepop_phf::OrderedSet;
    ///
    /// let mut builder = OrderedSet::<u32>::builder_with_capacity(100);
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
    /// use icepop_phf::OrderedSet;
    ///
    /// let mut builder = OrderedSet::<u32>::builder_with_capacity(100);
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
    /// use icepop_phf::OrderedSet;
    ///
    /// let mut builder = OrderedSet::<u32>::builder();
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
