use crate::portability::{NonPortable, OrderedMapOps, Portable};
use crate::table::{Builder as TableBuilder, Table};

use ::portable::DefaultHasherSeed;

#[doc(inline)]
pub use crate::map_iters::{
    IntoIter, IntoKeys, IntoValues, Iter, IterMut, Keys, Values, ValuesMut,
};

#[cfg(feature = "rkyv")]
pub mod rkyv;

mod non_portable;
mod portable;

pub struct OrderedMap<K, V, S = DefaultHasherSeed, P = NonPortable> {
    table: Table<OrderedMapOps<K, V>, S, P>,
}

pub struct Builder<K, V, S = DefaultHasherSeed, P = NonPortable> {
    builder: TableBuilder<OrderedMapOps<K, V>, S, P>,
}

pub type PortableOrderedMap<K, V, S = DefaultHasherSeed> = OrderedMap<K, V, S, Portable>;

pub type PortableBuilder<K, V, S = DefaultHasherSeed> = Builder<K, V, S, Portable>;

impl<K, V, S, P> OrderedMap<K, V, S, P> {
    pub fn hasher(&self) -> &S {
        self.table.hasher()
    }

    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            iter: self.table.iter(),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        IterMut {
            iter: self.table.iter_mut(),
        }
    }

    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys {
            iter: self.table.iter(),
        }
    }

    pub fn into_keys(self) -> IntoKeys<K, V> {
        IntoKeys {
            iter: self.table.into_iter(),
        }
    }

    pub fn values(&self) -> Values<'_, K, V> {
        Values {
            iter: self.table.iter(),
        }
    }

    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        ValuesMut {
            iter: self.table.iter_mut(),
        }
    }

    pub fn into_values(self) -> IntoValues<K, V> {
        IntoValues {
            iter: self.table.into_iter(),
        }
    }

    pub fn as_slice(&self) -> &[(K, V)] {
        self.table.as_slice()
    }

    pub fn index(&self, index: usize) -> Option<(&K, &V)> {
        self.table.map_index(index)
    }

    pub fn index_mut(&mut self, index: usize) -> Option<(&K, &mut V)> {
        self.table.map_index_mut(index)
    }

    pub unsafe fn index_unchecked(&self, index: usize) -> (&K, &V) {
        unsafe { self.table.map_index_unchecked(index) }
    }

    pub unsafe fn index_unchecked_mut(&mut self, index: usize) -> (&K, &mut V) {
        unsafe { self.table.map_index_unchecked_mut(index) }
    }
}

impl<K, V, S, P> core::fmt::Debug for OrderedMap<K, V, S, P>
where
    K: core::fmt::Debug,
    V: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<K, V, S, P> Clone for OrderedMap<K, V, S, P>
where
    K: Clone,
    V: Clone,
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            table: self.table.clone(),
        }
    }

    fn clone_from(&mut self, other: &Self) {
        self.table.clone_from(&other.table)
    }
}

impl<'a, K, V, S, P> IntoIterator for &'a OrderedMap<K, V, S, P> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V, S, P> IntoIterator for &'a mut OrderedMap<K, V, S, P> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<K, V, S, P> IntoIterator for OrderedMap<K, V, S, P> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            iter: self.table.into_iter(),
        }
    }
}

impl<K, V, S, P> Builder<K, V, S, P> {
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

    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            iter: self.builder.iter(),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        IterMut {
            iter: self.builder.iter_mut(),
        }
    }

    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys {
            iter: self.builder.iter(),
        }
    }

    pub fn into_keys(self) -> IntoKeys<K, V> {
        IntoKeys {
            iter: self.builder.into_iter(),
        }
    }

    pub fn values(&self) -> Values<'_, K, V> {
        Values {
            iter: self.builder.iter(),
        }
    }

    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        ValuesMut {
            iter: self.builder.iter_mut(),
        }
    }

    pub fn into_values(self) -> IntoValues<K, V> {
        IntoValues {
            iter: self.builder.into_iter(),
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

impl<K, V, S, P> core::fmt::Debug for Builder<K, V, S, P>
where
    K: core::fmt::Debug,
    V: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<K, V, S, P> Clone for Builder<K, V, S, P>
where
    K: Clone,
    V: Clone,
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

impl<'a, K, V, S, P> IntoIterator for &'a Builder<K, V, S, P> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V, S, P> IntoIterator for &'a mut Builder<K, V, S, P> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<K, V, S, P> IntoIterator for Builder<K, V, S, P> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            iter: self.builder.into_iter(),
        }
    }
}
