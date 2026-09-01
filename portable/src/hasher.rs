//! Hasher traits guaranteeing platform-independent hash values.

use core::hash::{BuildHasher, Hasher};

use crate::hash::PortableHash;

/// A [`Hasher`] whose output is identical on every platform.
///
/// Implementing this trait asserts that the hash depends only on the bytes written to the
/// hasher, and not on endianness, pointer width, operating system, or process-local state such
/// as a randomly generated seed. Hashers that do not satisfy this can still be used with
/// [`PortableHash`], but their results are then only comparable within a single process.
///
/// # Example
///
/// ```
/// use portable::{AssertPortable, PortableHash, PortableHasher};
/// use std::hash::{DefaultHasher, Hasher};
///
/// fn hash_of<H: PortableHasher>(value: &impl PortableHash, mut state: H) -> u64 {
///     value.portable_hash(&mut state);
///     state.finish()
/// }
///
/// // `AssertPortable` turns any hasher into a `PortableHasher`.
/// assert_eq!(
///     hash_of(&1usize, AssertPortable(DefaultHasher::new())),
///     hash_of(&1u64, AssertPortable(DefaultHasher::new())),
/// );
/// ```
pub trait PortableHasher: Hasher {}

/// Portable hashing helpers for [`BuildHasher`]s that build a [`PortableHasher`].
///
/// Blanket-implemented for every such [`BuildHasher`]; it is never implemented manually.
///
/// # Example
///
/// ```
/// use portable::{AssertPortable, PortableBuildHasher};
/// use std::hash::RandomState;
///
/// let build_hasher = AssertPortable(RandomState::new());
///
/// assert_eq!(build_hasher.portable_hash_one(&1u32), build_hasher.portable_hash_one(&1u32));
/// ```
pub trait PortableBuildHasher: BuildHasher<Hasher: PortableHasher> {
    /// Hashes one value with a freshly built hasher.
    ///
    /// # Example
    ///
    /// ```
    /// use portable::{AssertPortable, PortableBuildHasher};
    /// use std::hash::RandomState;
    ///
    /// let build_hasher = AssertPortable(RandomState::new());
    ///
    /// // Equal values hash equally, even across types.
    /// assert_eq!(
    ///     build_hasher.portable_hash_one(&1usize),
    ///     build_hasher.portable_hash_one(&1u64),
    /// );
    /// ```
    fn portable_hash_one<T>(&self, value: &T) -> u64
    where
        T: PortableHash + ?Sized,
    {
        let mut state = self.build_hasher();
        value.portable_hash(&mut state);
        state.finish()
    }
}

impl<S> PortableBuildHasher for S
where
    S: BuildHasher,
    S::Hasher: PortableHasher,
{
}

impl<H: Hasher> Hasher for crate::AssertPortable<H> {
    fn finish(&self) -> u64 {
        self.0.finish()
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0.write(bytes);
    }

    fn write_u8(&mut self, v: u8) {
        v.portable_hash(&mut self.0);
    }

    fn write_u16(&mut self, v: u16) {
        v.portable_hash(&mut self.0);
    }

    fn write_u32(&mut self, v: u32) {
        v.portable_hash(&mut self.0);
    }

    fn write_u64(&mut self, v: u64) {
        v.portable_hash(&mut self.0);
    }

    fn write_u128(&mut self, v: u128) {
        v.portable_hash(&mut self.0);
    }

    fn write_usize(&mut self, v: usize) {
        v.portable_hash(&mut self.0);
    }

    fn write_i8(&mut self, v: i8) {
        v.portable_hash(&mut self.0);
    }

    fn write_i16(&mut self, v: i16) {
        v.portable_hash(&mut self.0);
    }

    fn write_i32(&mut self, v: i32) {
        v.portable_hash(&mut self.0);
    }

    fn write_i64(&mut self, v: i64) {
        v.portable_hash(&mut self.0);
    }

    fn write_i128(&mut self, v: i128) {
        v.portable_hash(&mut self.0);
    }

    fn write_isize(&mut self, v: isize) {
        v.portable_hash(&mut self.0);
    }
}

impl<H: Hasher> PortableHasher for crate::AssertPortable<H> {}

impl<S: BuildHasher> BuildHasher for crate::AssertPortable<S> {
    type Hasher = crate::AssertPortable<S::Hasher>;

    fn build_hasher(&self) -> Self::Hasher {
        crate::AssertPortable(self.0.build_hasher())
    }
}
