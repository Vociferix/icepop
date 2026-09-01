//! Hashing and comparison traits that produce identical results on every platform.
//!
//! [`PortableHash`] feeds the same bytes to a hasher regardless of target endianness or
//! pointer width, and [`PortableEq`] and [`PortableOrd`] compare values the same way
//! everywhere. The comparison traits also compare *across* types: any two types that share an
//! underlying representation — a little-endian `u32` and a native `u32`, an array and a slice —
//! can be compared with each other. Fixed-width integers of different widths deliberately do
//! not compare; `usize` and `isize` are the exception, comparing against every width they may
//! be stored as, because that width is chosen by the serialization format rather than by the
//! value.
//!
//! # Contract
//!
//! Values that compare equal via [`PortableEq::portable_eq`] produce the same hash via
//! [`PortableHash::portable_hash`], even when their types differ. A [`PortableHasher`]
//! additionally guarantees that those hashes are identical across platforms. With any other
//! hasher, the contract holds only within a single process, for identical hashers.
//!
//! # How it works
//!
//! Comparison is derived from [`VisitPortableRepr`](repr::VisitPortableRepr), which names a
//! type's canonical representation and lends out a view of it; two types are comparable when
//! their representations are. [`PortableHash`] does not go through representations — each type
//! implements it directly, which allows type-specific optimizations.
//!
//! # Example
//!
//! ```
//! use portable::{AssertPortable, PortableBuildHasher, PortableEq, PortableOrd};
//! use std::hash::RandomState;
//!
//! // Different types, same representation.
//! assert!(1usize.portable_eq(&1u64));
//! assert!(2usize.portable_cmp(&10u64).is_lt());
//!
//! // Values that compare equal hash equally.
//! let build_hasher = AssertPortable(RandomState::new());
//! assert_eq!(
//!     build_hasher.portable_hash_one(&1usize),
//!     build_hasher.portable_hash_one(&1u64),
//! );
//! ```
//!
//! # Features
//!
//! - `alloc` *(default)*: Implements the traits for `alloc` types such as `Box`, `Vec`,
//!   `String` and `BTreeMap`. The crate is `no_std` either way.
//! - `default-hasher` *(default)*: Provides `DefaultHasher` and `DefaultHasherSeed`, a
//!   portable [rapidhash](https://docs.rs/rapidhash)-based hasher.
//! - `getrandom` *(default)*: Seeds `DefaultHasherSeed::new` from the operating system's
//!   entropy source instead of a compile-time random value.
//! - `serde`: Implements `Serialize` and `Deserialize` for `DefaultHasherSeed`.
//!
//! The remaining features implement the traits for the types of a third-party crate, and are
//! named after the crate and the version they support:
//!
//! - `allocator-api2-0_2`: [allocator-api2 0.2](https://docs.rs/allocator-api2/0.2/allocator_api2/)
//! - `allocator-api2-0_3`: [allocator-api2 0.3](https://docs.rs/allocator-api2/0.3/allocator_api2/)
//! - `allocator-api2-0_4`: [allocator-api2 0.4](https://docs.rs/allocator-api2/0.4/allocator_api2/)
//! - `arrayvec-0_7`: [arrayvec 0.7](https://docs.rs/arrayvec/0.7/arrayvec/)
//! - `ascii-1`: [ascii 1](https://docs.rs/ascii/1/ascii/)
//! - `bstr-1`: [bstr 1](https://docs.rs/bstr/1/bstr/)
//! - `bumpalo-3`: [bumpalo 3](https://docs.rs/bumpalo/3/bumpalo/)
//! - `bytes-1`: [bytes 1](https://docs.rs/bytes/1/bytes/)
//! - `either-1`: [either 1](https://docs.rs/either/1/either/)
//! - `rend-0_5`: [rend 0.5](https://docs.rs/rend/0.5/rend/)
//! - `rkyv-0_8`: [rkyv 0.8](https://docs.rs/rkyv/0.8/rkyv/). Implies `rend-0_5`.
//! - `smallvec-1`: [smallvec 1](https://docs.rs/smallvec/1/smallvec/)
//! - `smol_str-0_2`: [smol_str 0.2](https://docs.rs/smol_str/0.2/smol_str/)
//! - `smol_str-0_3`: [smol_str 0.3](https://docs.rs/smol_str/0.3/smol_str/)
//! - `thin-vec-0_2`: [thin-vec 0.2](https://docs.rs/thin-vec/0.2/thin_vec/)
//! - `tinyvec-1`: [tinyvec 1](https://docs.rs/tinyvec/1/tinyvec/)
//! - `triomphe-0_1`: [triomphe 0.1](https://docs.rs/triomphe/0.1/triomphe/)

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod eq;
pub mod hash;
pub mod hasher;
pub mod ord;
pub mod repr;

#[cfg(feature = "default-hasher")]
mod default_hasher;

#[doc(inline)]
pub use self::{
    eq::PortableEq,
    hash::PortableHash,
    hasher::{PortableBuildHasher, PortableHasher},
    ord::PortableOrd,
};

#[cfg(feature = "default-hasher")]
pub use default_hasher::{DefaultHasher, DefaultHasherSeed};

#[cfg(all(feature = "default-hasher", feature = "rkyv-0_8"))]
pub use default_hasher::ArchivedDefaultHasherSeed;

/// Wrapper that gives a value the standard traits in terms of the portable ones.
///
/// [`Hash`](core::hash::Hash), [`Eq`] and [`Ord`] on `Portable<T>` forward to [`PortableHash`],
/// [`PortableEq`] and [`PortableOrd`] on `T`, so portable behavior can be used with APIs that
/// require the standard traits. Wrappers around different types compare with each other
/// whenever the types they wrap do.
///
/// # Example
///
/// ```
/// use portable::Portable;
///
/// assert_eq!(Portable(1usize), Portable(1u64));
/// assert!(Portable(2usize) < Portable(10u64));
/// ```
#[derive(Debug, Clone, Copy, Default)]
#[repr(transparent)]
pub struct Portable<T: ?Sized>(pub T);

/// Wrapper that asserts a value's standard traits are already portable.
///
/// [`PortableHash`], [`PortableEq`] and [`PortableOrd`] on `AssertPortable<T>` forward to
/// [`Hash`](core::hash::Hash), [`Eq`] and [`Ord`] on `T`, and a wrapped hasher becomes a
/// [`PortableHasher`]. Use it for types whose standard implementations are known to behave
/// identically everywhere, or where portability is not actually required — for instance to
/// hash with a process-local hasher, whose results are stable within one process only.
///
/// # Example
///
/// ```
/// use portable::{AssertPortable, PortableBuildHasher};
/// use std::hash::RandomState;
///
/// let build_hasher = AssertPortable(RandomState::new());
/// let value = AssertPortable("hello");
///
/// assert_eq!(
///     build_hasher.portable_hash_one(&value),
///     build_hasher.portable_hash_one(&value),
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct AssertPortable<T: ?Sized>(pub T);

impl<T: ?Sized> Portable<T> {
    /// Wraps a reference, including a reference to an unsized value.
    ///
    /// # Example
    ///
    /// ```
    /// use portable::Portable;
    ///
    /// assert_eq!(Portable::from_ref("hello"), Portable::from_ref("hello"));
    /// ```
    pub const fn from_ref(value: &T) -> &Self {
        // SAFETY: `Portable<T>` is `repr(transparent)` over `T`, so the cast preserves the
        // pointer's address, provenance and metadata, and every valid `&T` is a valid `&Self`.
        unsafe { &*(value as *const T as *const Self) }
    }

    /// Wraps a mutable reference, including a reference to an unsized value.
    ///
    /// # Example
    ///
    /// ```
    /// use portable::Portable;
    ///
    /// let mut value = 1u32;
    /// Portable::from_mut(&mut value).0 = 2;
    ///
    /// assert_eq!(value, 2);
    /// ```
    pub const fn from_mut(value: &mut T) -> &mut Self {
        // SAFETY: `Portable<T>` is `repr(transparent)` over `T`, so the cast preserves the
        // pointer's address, provenance and metadata, and the unique borrow of `value` is
        // passed on to the result.
        unsafe { &mut *(value as *mut T as *mut Self) }
    }
}

impl<T: ?Sized> core::borrow::Borrow<T> for Portable<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}

impl<T: ?Sized> core::borrow::BorrowMut<T> for Portable<T> {
    fn borrow_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: ?Sized> AsRef<T> for Portable<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T: ?Sized> AsMut<T> for Portable<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: ?Sized> core::ops::Deref for Portable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: ?Sized> core::ops::DerefMut for Portable<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: ?Sized> AssertPortable<T> {
    /// Wraps a reference, including a reference to an unsized value.
    ///
    /// # Example
    ///
    /// ```
    /// use portable::AssertPortable;
    ///
    /// assert_eq!(AssertPortable::from_ref("hello"), AssertPortable::from_ref("hello"));
    /// ```
    pub const fn from_ref(value: &T) -> &Self {
        // SAFETY: `AssertPortable<T>` is `repr(transparent)` over `T`, so the cast preserves the
        // pointer's address, provenance and metadata, and every valid `&T` is a valid `&Self`.
        unsafe { &*(value as *const T as *const Self) }
    }

    /// Wraps a mutable reference, including a reference to an unsized value.
    ///
    /// # Example
    ///
    /// ```
    /// use portable::AssertPortable;
    ///
    /// let mut value = 1u32;
    /// AssertPortable::from_mut(&mut value).0 = 2;
    ///
    /// assert_eq!(value, 2);
    /// ```
    pub const fn from_mut(value: &mut T) -> &mut Self {
        // SAFETY: `AssertPortable<T>` is `repr(transparent)` over `T`, so the cast preserves the
        // pointer's address, provenance and metadata, and the unique borrow of `value` is
        // passed on to the result.
        unsafe { &mut *(value as *mut T as *mut Self) }
    }
}

impl<T: ?Sized> core::borrow::Borrow<T> for AssertPortable<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}

impl<T: ?Sized> core::borrow::BorrowMut<T> for AssertPortable<T> {
    fn borrow_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: ?Sized> AsRef<T> for AssertPortable<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T: ?Sized> AsMut<T> for AssertPortable<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: ?Sized> core::ops::Deref for AssertPortable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: ?Sized> core::ops::DerefMut for AssertPortable<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
