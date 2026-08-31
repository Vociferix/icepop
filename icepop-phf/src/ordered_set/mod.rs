use crate::portability::{NonPortable, OrderedSetOps, Portable};
use crate::table::{Builder as TableBuilder, Table};

use ::portable::DefaultHasherSeed;

#[doc(inline)]
pub use crate::set_iters::{IntoIter, Iter};

#[cfg(feature = "rkyv")]
pub mod rkyv;

mod non_portable;
mod portable;

pub struct OrderedSet<T, S = DefaultHasherSeed, P = NonPortable> {
    table: Table<OrderedSetOps<T>, S, P>,
}

pub struct Builder<T, S = DefaultHasherSeed, P = NonPortable> {
    builder: TableBuilder<OrderedSetOps<T>, S, P>,
}

pub type PortableOrderedSet<T, S = DefaultHasherSeed> = OrderedSet<T, S, Portable>;

pub type PortableBuilder<T, S = DefaultHasherSeed> = Builder<T, S, Portable>;

impl<T, S, P> OrderedSet<T, S, P> {
    pub fn hasher(&self) -> &S {
        self.table.hasher()
    }

    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            iter: self.table.iter(),
        }
    }

    pub fn as_slice(&self) -> &[T] {
        self.table.as_slice()
    }

    pub fn index(&self, index: usize) -> Option<&T> {
        self.table.index(index)
    }

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
    pub fn hasher(&self) -> &S {
        self.builder.hasher()
    }

    pub fn is_empty(&self) -> bool {
        self.builder.is_empty()
    }

    pub fn len(&self) -> usize {
        self.builder.len()
    }

    pub fn capacity(&self) -> usize {
        self.builder.capacity()
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            iter: self.builder.iter(),
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        self.builder.reserve(additional);
    }

    pub fn shrink_to_fit(&mut self) {
        self.builder.shrink_to_fit();
    }

    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.builder.shrink_to(min_capacity);
    }

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
