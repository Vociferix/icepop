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

#[cfg(test)]
mod tests {
    use super::{PortableBuildHasher, PortableHasher};

    use crate::AssertPortable;
    use crate::hash::PortableHash;

    use core::hash::{BuildHasher, Hasher};

    /// Hasher that records the bytes written to it, so writes can be compared exactly.
    #[derive(Default)]
    struct Recorder(std::vec::Vec<u8>);

    impl Hasher for Recorder {
        fn finish(&self) -> u64 {
            0
        }

        fn write(&mut self, bytes: &[u8]) {
            self.0.extend_from_slice(bytes);
        }
    }

    fn written(f: impl FnOnce(&mut AssertPortable<Recorder>)) -> std::vec::Vec<u8> {
        let mut state = AssertPortable(Recorder::default());
        f(&mut state);
        state.0.0
    }

    #[test]
    fn wrapping_a_hasher_normalizes_integer_writes_to_eight_little_endian_bytes() {
        let expected = 1u64.to_le_bytes().to_vec();

        assert_eq!(written(|state| state.write_u16(1)), expected);
        assert_eq!(written(|state| state.write_u32(1)), expected);
        assert_eq!(written(|state| state.write_u64(1)), expected);
        assert_eq!(written(|state| state.write_usize(1)), expected);

        let expected = 1i64.to_le_bytes().to_vec();

        assert_eq!(written(|state| state.write_i16(1)), expected);
        assert_eq!(written(|state| state.write_i32(1)), expected);
        assert_eq!(written(|state| state.write_i64(1)), expected);
        assert_eq!(written(|state| state.write_isize(1)), expected);

        // Negative values must sign-extend rather than be truncated.
        assert_eq!(
            written(|state| state.write_i16(-1)),
            (-1i64).to_le_bytes().to_vec()
        );

        assert_eq!(
            written(|state| state.write_u128(1)),
            1u128.to_le_bytes().to_vec()
        );
        assert_eq!(written(|state| state.write_u8(1)), std::vec![1]);
        assert_eq!(written(|state| state.write_i8(-1)), std::vec![0xff]);
    }

    #[test]
    fn wrapping_a_hasher_passes_byte_writes_through_untouched() {
        assert_eq!(written(|state| state.write(b"abc")), b"abc".to_vec());
    }

    #[test]
    fn a_wrapped_hasher_hashes_the_same_bytes_as_the_portable_impl() {
        let via_hasher = written(|state| state.write_u32(7));
        let via_trait = {
            let mut state = AssertPortable(Recorder::default());
            7u32.portable_hash(&mut state);
            state.0.0
        };

        assert_eq!(via_hasher, via_trait);
    }

    #[test]
    fn hash_one_matches_building_and_finishing_by_hand() {
        let build_hasher = AssertPortable(std::hash::RandomState::new());

        let by_hand = {
            let mut state = build_hasher.build_hasher();
            "hello".portable_hash(&mut state);
            state.finish()
        };

        assert_eq!(build_hasher.portable_hash_one("hello"), by_hand);
    }

    #[test]
    fn a_wrapped_build_hasher_builds_a_portable_hasher() {
        fn require_portable<H: PortableHasher>(_: &H) {}

        let build_hasher = AssertPortable(std::hash::RandomState::new());

        require_portable(&build_hasher.build_hasher());
    }
}
