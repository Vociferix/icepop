//! [`PortableHash`], the platform-independent counterpart of [`Hash`](core::hash::Hash).

use core::hash::Hasher;
use core::ops::RangeBounds;

/// Derives [`PortableHash`](trait@PortableHash), hashing each field in turn.
///
/// Hashing does not go through representations: the impl is written against the type itself,
/// and hashes every field in declaration order. An enum first hashes its variant's index, as
/// the smallest unsigned integer that can hold every index, and then that variant's fields.
///
/// Because equal values must hash equally, a type whose fields hash portably will agree with
/// any type it compares equal to — including its archived counterpart under `rkyv`.
///
/// # Attributes
///
/// Attributes are written `#[portable(...)]` on the type. Each accepts `name = value` and
/// `name(value)` alike.
///
/// - `rkyv` — also implement the trait for the type's [rkyv](https://docs.rs/rkyv) archived
///   counterpart, so a value and its archived form hash identically. The archived type is
///   taken from `#[rkyv(archived = ...)]` when present and is `Archived{Type}` otherwise;
///   `rkyv = Name` overrides both.
/// - `rkyv_crate = path` — path to the `rkyv` crate. Defaults to `::rkyv`.
/// - `hash_bounds(...)` — where predicates for the generated impl, in place of the default
///   bound of [`PortableHash`] on each of the type's parameters. `bounds(...)` sets them for
///   every `portable` derive at once, and the more specific attribute wins. Bounds declared on
///   the type's own parameters are always kept.
/// - `crate = path` — path to the `portable` crate. Defaults to `::portable`.
///
/// # Example
///
/// ```
/// use portable::{AssertPortable, PortableBuildHasher, PortableHash};
/// use std::hash::RandomState;
///
/// #[derive(PortableHash)]
/// struct Point {
///     x: u32,
///     y: u64,
/// }
///
/// let build_hasher = AssertPortable(RandomState::new());
///
/// assert_eq!(
///     build_hasher.portable_hash_one(&Point { x: 1, y: 2 }),
///     build_hasher.portable_hash_one(&Point { x: 1, y: 2 }),
/// );
/// ```
///
/// # Generated impl
///
/// The `Point` above generates approximately:
///
/// ```ignore
/// impl PortableHash for Point {
///     fn portable_hash<H>(&self, state: &mut H)
///     where
///         H: Hasher,
///     {
///         self.x.portable_hash(state);
///         self.y.portable_hash(state);
///     }
/// }
/// ```
///
/// With `rkyv`, the identical body is also emitted as `impl PortableHash for ArchivedPoint`,
/// which is what makes a value and its archived form hash equally.
#[cfg(feature = "derive")]
pub use portable_derive::PortableHash;

/// A type that can be hashed identically on every platform.
///
/// Unlike [`Hash`](core::hash::Hash), implementations must write the same bytes regardless of
/// target endianness or pointer width, so hashing a value with a
/// [`PortableHasher`](crate::PortableHasher) yields the same hash everywhere. Each type
/// implements this trait directly rather than through a shared representation, which leaves
/// room for type-specific optimizations.
///
/// Implementations must uphold the contract with [`PortableEq`](crate::PortableEq): values
/// that compare equal — including values of different types — must hash equally.
///
/// # Example
///
/// ```
/// use portable::{AssertPortable, PortableBuildHasher, PortableHash};
/// use std::hash::{Hasher, RandomState};
///
/// struct Point {
///     x: i32,
///     y: i32,
/// }
///
/// impl PortableHash for Point {
///     fn portable_hash<H: Hasher>(&self, state: &mut H) {
///         self.x.portable_hash(state);
///         self.y.portable_hash(state);
///     }
/// }
///
/// let build_hasher = AssertPortable(RandomState::new());
///
/// assert_eq!(
///     build_hasher.portable_hash_one(&Point { x: 1, y: 2 }),
///     build_hasher.portable_hash_one(&Point { x: 1, y: 2 }),
/// );
/// ```
pub trait PortableHash {
    /// Feeds this value into the given hasher.
    ///
    /// # Example
    ///
    /// ```
    /// use portable::{AssertPortable, PortableHash};
    /// use std::hash::{DefaultHasher, Hasher};
    ///
    /// let mut state = AssertPortable(DefaultHasher::new());
    /// "hello".portable_hash(&mut state);
    ///
    /// assert_ne!(state.finish(), 0);
    /// ```
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher;

    /// Feeds a slice of values into the given hasher.
    ///
    /// Overriding this is an optimization only: it must hash the same bytes as calling
    /// [`portable_hash`](PortableHash::portable_hash) on each element in turn. It does not
    /// hash the length, so callers that need slices of different lengths to hash differently
    /// must hash the length themselves.
    ///
    /// # Example
    ///
    /// ```
    /// use portable::{AssertPortable, PortableHash};
    /// use std::hash::{DefaultHasher, Hasher};
    ///
    /// let mut slice_state = AssertPortable(DefaultHasher::new());
    /// u32::portable_hash_slice(&[1, 2, 3], &mut slice_state);
    ///
    /// let mut item_state = AssertPortable(DefaultHasher::new());
    /// for value in [1u32, 2, 3] {
    ///     value.portable_hash(&mut item_state);
    /// }
    ///
    /// assert_eq!(slice_state.finish(), item_state.finish());
    /// ```
    fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
    where
        Self: Sized,
        H: Hasher,
    {
        slice.iter().for_each(|item| item.portable_hash(state));
    }
}

impl<T> PortableHash for crate::Portable<T>
where
    T: PortableHash,
{
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.0.portable_hash(state);
    }

    fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
    where
        H: Hasher,
    {
        // SAFETY: `Portable<T>` is `repr(transparent)` over `T`, so `[Self]` has the
        // same layout as `[T]`.
        T::portable_hash_slice(unsafe { &*(slice as *const [Self] as *const [T]) }, state);
    }
}

impl<T> core::hash::Hash for crate::Portable<T>
where
    T: PortableHash,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.portable_hash(state);
    }

    fn hash_slice<H>(slice: &[Self], state: &mut H)
    where
        H: Hasher,
    {
        Self::portable_hash_slice(slice, state);
    }
}

impl<T> PortableHash for crate::AssertPortable<T>
where
    T: core::hash::Hash,
{
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.0.hash(state);
    }
}

/// Views a value as its raw bytes.
///
/// The bytes are hashed exactly as they are laid out in memory, so callers are also responsible
/// for only using this where that layout is the same on every platform.
///
/// # Safety
///
/// `T` must have no padding and no uninitialized bytes, since every byte of `val` is read.
#[cfg(feature = "rend-0_5")]
unsafe fn as_bytes<T>(val: &T) -> &[u8] {
    // SAFETY: the caller guarantees that every byte of `val` is initialized. The returned slice
    // borrows `val`, so the bytes stay valid and unmodified for its lifetime, and a value is
    // never larger than `isize::MAX` bytes.
    unsafe {
        core::slice::from_raw_parts(val as *const T as *const u8, core::mem::size_of_val(val))
    }
}

/// Views a slice as its raw bytes.
///
/// The bytes are hashed exactly as they are laid out in memory, so callers are also responsible
/// for only using this where that layout is the same on every platform.
///
/// # Safety
///
/// `T` must have no padding and no uninitialized bytes, since every byte of `slice` is read.
unsafe fn slice_as_bytes<T>(slice: &[T]) -> &[u8] {
    // SAFETY: the caller guarantees that every byte of every element is initialized, and array
    // elements are laid out contiguously with no padding between them. The returned slice
    // borrows `slice`, so the bytes stay valid and unmodified for its lifetime, and a slice is
    // never larger than `isize::MAX` bytes.
    unsafe { core::slice::from_raw_parts(slice.as_ptr().cast(), core::mem::size_of_val(slice)) }
}

impl<T> PortableHash for &T
where
    T: PortableHash + ?Sized,
{
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        T::portable_hash(*self, state);
    }
}

impl<T> PortableHash for crate::repr::Field<T>
where
    T: PortableHash + ?Sized,
{
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        T::portable_hash(self, state);
    }
}

impl<T> PortableHash for &mut T
where
    T: PortableHash + ?Sized,
{
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        T::portable_hash(*self, state);
    }
}

impl PortableHash for bool {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(&[u8::from(*self)]);
    }

    fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
    where
        H: Hasher,
    {
        // SAFETY: `bool` occupies a single initialized byte with no padding, so every byte of the slice is
        // initialized.
        state.write(unsafe { slice_as_bytes(slice) });
    }
}

impl PortableHash for u8 {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(&[*self]);
    }

    fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
    where
        H: Hasher,
    {
        state.write(slice);
    }
}

impl PortableHash for u16 {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(&(*self as u64).to_le_bytes());
    }
}

impl PortableHash for u32 {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(&(*self as u64).to_le_bytes());
    }
}

impl PortableHash for u64 {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(&self.to_le_bytes());
    }

    #[cfg(target_endian = "little")]
    fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
    where
        H: Hasher,
    {
        // SAFETY: `u64` has no padding, so every byte of the slice is
        // initialized.
        state.write(unsafe { slice_as_bytes(slice) });
    }
}

impl PortableHash for u128 {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(&self.to_le_bytes());
    }

    #[cfg(target_endian = "little")]
    fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
    where
        H: Hasher,
    {
        // SAFETY: `u128` has no padding, so every byte of the slice is
        // initialized.
        state.write(unsafe { slice_as_bytes(slice) });
    }
}

impl PortableHash for usize {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(&(*self as u64).to_le_bytes());
    }

    #[cfg(all(target_endian = "little", target_pointer_width = "64"))]
    fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
    where
        H: Hasher,
    {
        // SAFETY: `usize` has no padding, so every byte of the slice is
        // initialized.
        state.write(unsafe { slice_as_bytes(slice) });
    }
}

impl PortableHash for i8 {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(&[(*self).cast_unsigned()]);
    }

    fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
    where
        H: Hasher,
    {
        // SAFETY: `i8` has no padding, so every byte of the slice is
        // initialized.
        state.write(unsafe { slice_as_bytes(slice) });
    }
}

impl PortableHash for i16 {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(&(*self as i64).to_le_bytes());
    }
}

impl PortableHash for i32 {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(&(*self as i64).to_le_bytes());
    }
}

impl PortableHash for i64 {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(&self.to_le_bytes());
    }

    #[cfg(target_endian = "little")]
    fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
    where
        H: Hasher,
    {
        // SAFETY: `i64` has no padding, so every byte of the slice is
        // initialized.
        state.write(unsafe { slice_as_bytes(slice) });
    }
}

impl PortableHash for i128 {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(&self.to_le_bytes());
    }

    #[cfg(target_endian = "little")]
    fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
    where
        H: Hasher,
    {
        // SAFETY: `i128` has no padding, so every byte of the slice is
        // initialized.
        state.write(unsafe { slice_as_bytes(slice) });
    }
}

impl PortableHash for isize {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(&(*self as i64).to_le_bytes());
    }

    #[cfg(all(target_endian = "little", target_pointer_width = "64"))]
    fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
    where
        H: Hasher,
    {
        // SAFETY: `isize` has no padding, so every byte of the slice is
        // initialized.
        state.write(unsafe { slice_as_bytes(slice) });
    }
}

impl PortableHash for char {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(&(*self as u32).to_le_bytes());
    }

    #[cfg(target_endian = "little")]
    fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
    where
        H: Hasher,
    {
        // SAFETY: `char` is four bytes wide with no padding, so every byte of the slice is
        // initialized.
        state.write(unsafe { slice_as_bytes(slice) });
    }
}

impl PortableHash for core::num::NonZeroU8 {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(&[(*self).get()]);
    }

    fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
    where
        H: Hasher,
    {
        // SAFETY: `NonZeroU8` has the same layout as `u8`, which has no padding, so every byte of the slice is
        // initialized.
        state.write(unsafe { slice_as_bytes(slice) });
    }
}

impl PortableHash for core::num::NonZeroU16 {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(&((*self).get() as u64).to_le_bytes());
    }
}

impl PortableHash for core::num::NonZeroU32 {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(&((*self).get() as u64).to_le_bytes());
    }
}

impl PortableHash for core::num::NonZeroU64 {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(&(*self).get().to_le_bytes());
    }

    #[cfg(target_endian = "little")]
    fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
    where
        H: Hasher,
    {
        // SAFETY: `NonZeroU64` has the same layout as `u64`, which has no padding, so every byte of the slice is
        // initialized.
        state.write(unsafe { slice_as_bytes(slice) });
    }
}

impl PortableHash for core::num::NonZeroU128 {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(&(*self).get().to_le_bytes());
    }

    #[cfg(target_endian = "little")]
    fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
    where
        H: Hasher,
    {
        // SAFETY: `NonZeroU128` has the same layout as `u128`, which has no padding, so every byte of the slice is
        // initialized.
        state.write(unsafe { slice_as_bytes(slice) });
    }
}

impl PortableHash for core::num::NonZeroUsize {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(&((*self).get() as u64).to_le_bytes());
    }

    #[cfg(all(target_endian = "little", target_pointer_width = "64"))]
    fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
    where
        H: Hasher,
    {
        // SAFETY: `NonZeroUsize` has the same layout as `usize`, which has no padding, so every byte of the slice is
        // initialized.
        state.write(unsafe { slice_as_bytes(slice) });
    }
}

impl PortableHash for core::num::NonZeroI8 {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(&[(*self).get().cast_unsigned()]);
    }

    fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
    where
        H: Hasher,
    {
        // SAFETY: `NonZeroI8` has the same layout as `i8`, which has no padding, so every byte of the slice is
        // initialized.
        state.write(unsafe { slice_as_bytes(slice) });
    }
}

impl PortableHash for core::num::NonZeroI16 {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(&((*self).get() as i64).to_le_bytes());
    }
}

impl PortableHash for core::num::NonZeroI32 {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(&((*self).get() as i64).to_le_bytes());
    }
}

impl PortableHash for core::num::NonZeroI64 {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(&(*self).get().to_le_bytes());
    }

    #[cfg(target_endian = "little")]
    fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
    where
        H: Hasher,
    {
        // SAFETY: `NonZeroI64` has the same layout as `i64`, which has no padding, so every byte of the slice is
        // initialized.
        state.write(unsafe { slice_as_bytes(slice) });
    }
}

impl PortableHash for core::num::NonZeroI128 {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(&(*self).get().to_le_bytes());
    }

    #[cfg(target_endian = "little")]
    fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
    where
        H: Hasher,
    {
        // SAFETY: `NonZeroI128` has the same layout as `i128`, which has no padding, so every byte of the slice is
        // initialized.
        state.write(unsafe { slice_as_bytes(slice) });
    }
}

impl PortableHash for core::num::NonZeroIsize {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(&((*self).get() as i64).to_le_bytes());
    }

    #[cfg(all(target_endian = "little", target_pointer_width = "64"))]
    fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
    where
        H: Hasher,
    {
        // SAFETY: `NonZeroIsize` has the same layout as `isize`, which has no padding, so every byte of the slice is
        // initialized.
        state.write(unsafe { slice_as_bytes(slice) });
    }
}

impl PortableHash for () {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        let _ = state;
    }

    fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
    where
        H: Hasher,
    {
        let _ = (slice, state);
    }
}

macro_rules! tuple {
    ($t0:ident $(: $r0:ident)? $(, $tn:ident $(: $rn:ident)?)* $(,)?) => {
        tuple! { { [$($r0)?] $t0 } { $(([$($rn)?] $tn))* } }
    };
    ({ [$($r:ident)?] $($t:ident)* } { ([$($r0:ident)?] $t0:ident) $(([$($rn:ident)?] $tn:ident))* }) => {
        tuple! { { [$($r)?] $($t)* } { } }
        tuple! { { [$($r0)?] $($t)* $t0 } { $(([$($rn)?] $tn))* } }
    };
    ({ [$($r:ident)?] $($t:ident)* } { }) => {
        impl<$($t),*> PortableHash for ($($t,)*)
        where
            $($t: PortableHash,)*
        {
            fn portable_hash<H>(&self, state: &mut H)
            where
                H: Hasher,
            {
                #[allow(non_snake_case)]
                fn hash<$($t,)* H>(
                    ($($t,)*): &($($t,)*),
                    state: &mut H,
                )
                where
                    $($t: PortableHash,)*
                    H: Hasher,
                {
                    $($t.portable_hash(state);)*
                }

                hash(self, state);
            }
        }

        tuple! { @rkyv [$($r)?] $($t)* }
    };
    (@rkyv [$r:ident] $($t:ident)*) => {
        #[cfg(feature = "rkyv-0_8")]
        impl<$($t),*> PortableHash for rkyv_0_8::tuple::$r<$($t),*>
        where
            $($t: PortableHash,)*
        {
            fn portable_hash<H>(&self, state: &mut H)
            where
                H: Hasher,
            {
                #[allow(non_snake_case)]
                fn hash<$($t,)* H>(
                    rkyv_0_8::tuple::$r($($t),*): &rkyv_0_8::tuple::$r<$($t),*>,
                    state: &mut H,
                )
                where
                    $($t: PortableHash,)*
                    H: Hasher,
                {
                    $($t.portable_hash(state);)*
                }

                hash(self, state);
            }
        }
    };
    (@rkyv [] $($t:ident)*) => {};
}

tuple! {
    T0: ArchivedTuple1,
    T1: ArchivedTuple2,
    T2: ArchivedTuple3,
    T3: ArchivedTuple4,
    T4: ArchivedTuple5,
    T5: ArchivedTuple6,
    T6: ArchivedTuple7,
    T7: ArchivedTuple8,
    T8: ArchivedTuple9,
    T9: ArchivedTuple10,
    T10: ArchivedTuple11,
    T11: ArchivedTuple12,
    T12: ArchivedTuple13,
    T13,
    T14,
    T15,
}

impl<T> PortableHash for [T]
where
    T: PortableHash,
{
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.len().portable_hash(state);
        T::portable_hash_slice(self, state);
    }
}

impl<T, const N: usize> PortableHash for [T; N]
where
    T: PortableHash,
{
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        let slice: &[T] = self;
        slice.portable_hash(state);
    }
}

impl PortableHash for str {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(self.as_bytes());
        state.write(b"\xff");
    }
}

impl PortableHash for core::ffi::CStr {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(self.to_bytes_with_nul());
    }
}

impl PortableHash for core::convert::Infallible {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        let _ = state;
    }

    fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
    where
        H: Hasher,
    {
        let _ = (slice, state);
    }
}

impl<T: ?Sized> PortableHash for core::marker::PhantomData<T> {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        let _ = state;
    }

    fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
    where
        H: Hasher,
    {
        let _ = (slice, state);
    }
}

impl PortableHash for core::marker::PhantomPinned {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        let _ = state;
    }

    fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
    where
        H: Hasher,
    {
        let _ = (slice, state);
    }
}

impl<T> PortableHash for core::panic::AssertUnwindSafe<T>
where
    T: PortableHash,
{
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.0.portable_hash(state);
    }
}

impl PortableHash for core::cmp::Ordering {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        use core::cmp::Ordering::{Equal, Greater, Less};

        (match *self {
            Less => -1i8,
            Equal => 0i8,
            Greater => 1i8,
        })
        .portable_hash(state);
    }
}

impl<T> PortableHash for core::cmp::Reverse<T>
where
    T: PortableHash,
{
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.0.portable_hash(state);
    }

    fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
    where
        H: Hasher,
    {
        // SAFETY: `Reverse<T>` is `repr(transparent)` over `T`, so `[Self]` has the
        // same layout as `[T]`.
        T::portable_hash_slice(unsafe { &*(slice as *const [Self] as *const [T]) }, state);
    }
}

impl<T> PortableHash for core::num::Saturating<T>
where
    T: PortableHash,
{
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.0.portable_hash(state);
    }

    fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
    where
        H: Hasher,
    {
        // SAFETY: `Saturating<T>` is `repr(transparent)` over `T`, so `[Self]` has the
        // same layout as `[T]`.
        T::portable_hash_slice(unsafe { &*(slice as *const [Self] as *const [T]) }, state);
    }
}

impl<T> PortableHash for core::num::Wrapping<T>
where
    T: PortableHash,
{
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.0.portable_hash(state);
    }

    fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
    where
        H: Hasher,
    {
        // SAFETY: `Wrapping<T>` is `repr(transparent)` over `T`, so `[Self]` has the
        // same layout as `[T]`.
        T::portable_hash_slice(unsafe { &*(slice as *const [Self] as *const [T]) }, state);
    }
}

impl<T> PortableHash for Option<T>
where
    T: PortableHash,
{
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        match self {
            Some(val) => (true, val).portable_hash(state),
            None => false.portable_hash(state),
        }
    }
}

impl<T, E> PortableHash for Result<T, E>
where
    T: PortableHash,
    E: PortableHash,
{
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        match self {
            Ok(val) => (false, val).portable_hash(state),
            Err(err) => (true, err).portable_hash(state),
        }
    }
}

impl<T> PortableHash for core::ops::Bound<T>
where
    T: PortableHash,
{
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        use core::ops::Bound;

        match self {
            Bound::Unbounded => 0u8.portable_hash(state),
            Bound::Included(val) => (1u8, val).portable_hash(state),
            Bound::Excluded(val) => (2u8, val).portable_hash(state),
        }
    }
}

impl<T> PortableHash for core::ops::Range<T>
where
    T: PortableHash,
{
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        (self.start_bound(), self.end_bound()).portable_hash(state);
    }
}

impl<T> PortableHash for core::ops::RangeInclusive<T>
where
    T: PortableHash,
{
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        (self.start_bound(), self.end_bound()).portable_hash(state);
    }
}

impl<T> PortableHash for core::ops::RangeFrom<T>
where
    T: PortableHash,
{
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        (self.start_bound(), self.end_bound()).portable_hash(state);
    }
}

impl PortableHash for core::ops::RangeFull {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        fn get_bounds<R>(
            range: &R,
        ) -> (
            core::ops::Bound<core::convert::Infallible>,
            core::ops::Bound<core::convert::Infallible>,
        )
        where
            R: RangeBounds<core::convert::Infallible>,
        {
            (range.start_bound().cloned(), range.end_bound().cloned())
        }

        get_bounds(self).portable_hash(state);
    }

    fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
    where
        H: Hasher,
    {
        fn get_bounds<R>(
            range: &R,
        ) -> (
            core::ops::Bound<core::convert::Infallible>,
            core::ops::Bound<core::convert::Infallible>,
        )
        where
            R: RangeBounds<core::convert::Infallible>,
        {
            (range.start_bound().cloned(), range.end_bound().cloned())
        }

        let bounds = get_bounds(&..);

        (0..slice.len()).for_each(move |_| bounds.portable_hash(state));
    }
}

impl<T> PortableHash for core::ops::RangeTo<T>
where
    T: PortableHash,
{
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        (self.start_bound(), self.end_bound()).portable_hash(state);
    }
}

impl<T> PortableHash for core::ops::RangeToInclusive<T>
where
    T: PortableHash,
{
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        (self.start_bound(), self.end_bound()).portable_hash(state);
    }
}

impl<T> PortableHash for core::range::Range<T>
where
    T: PortableHash,
{
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        (self.start_bound(), self.end_bound()).portable_hash(state);
    }
}

impl<T> PortableHash for core::range::RangeInclusive<T>
where
    T: PortableHash,
{
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        (self.start_bound(), self.end_bound()).portable_hash(state);
    }
}

impl<T> PortableHash for core::range::RangeFrom<T>
where
    T: PortableHash,
{
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        (self.start_bound(), self.end_bound()).portable_hash(state);
    }
}

impl<T> PortableHash for core::range::RangeToInclusive<T>
where
    T: PortableHash,
{
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        (self.start_bound(), self.end_bound()).portable_hash(state);
    }
}

impl PortableHash for core::net::Ipv4Addr {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        false.portable_hash(state);
        state.write(&self.octets());
    }
}

impl PortableHash for core::net::Ipv6Addr {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        true.portable_hash(state);
        state.write(&self.octets());
    }
}

impl PortableHash for core::net::IpAddr {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        match self {
            Self::V4(ip) => ip.portable_hash(state),
            Self::V6(ip) => ip.portable_hash(state),
        }
    }
}

impl PortableHash for core::net::SocketAddrV4 {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        (self.ip(), self.port()).portable_hash(state);
    }
}

impl PortableHash for core::net::SocketAddrV6 {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        (self.ip(), self.port()).portable_hash(state);
        // Both fields sit at a fixed position in a fixed-width encoding, so they are written
        // at their own width rather than widened the way a bare `u32` would be.
        state.write(&self.flowinfo().to_le_bytes());
        state.write(&self.scope_id().to_le_bytes());
    }
}

impl PortableHash for core::net::SocketAddr {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        match self {
            Self::V4(addr) => addr.portable_hash(state),
            Self::V6(addr) => addr.portable_hash(state),
        }
    }
}

impl<P> PortableHash for core::pin::Pin<P>
where
    P: core::ops::Deref,
    P::Target: PortableHash,
{
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        <P::Target as PortableHash>::portable_hash(self, state);
    }
}

impl<T> PortableHash for core::task::Poll<T>
where
    T: PortableHash,
{
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        match self {
            Self::Ready(val) => (true, val).portable_hash(state),
            Self::Pending => false.portable_hash(state),
        }
    }
}

impl PortableHash for core::time::Duration {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.as_secs().portable_hash(state);
        self.subsec_nanos().portable_hash(state);
    }
}

cfg_select!(feature = "alloc" => {
    impl<T> PortableHash for alloc::boxed::Box<T>
    where
        T: PortableHash + ?Sized,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            T::portable_hash(self, state);
        }
    }

    impl<T> PortableHash for alloc::rc::Rc<T>
    where
        T: PortableHash + ?Sized,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            T::portable_hash(self, state);
        }
    }

    impl<T> PortableHash for alloc::sync::Arc<T>
    where
        T: PortableHash + ?Sized,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            T::portable_hash(self, state);
        }
    }

    impl<T> PortableHash for alloc::vec::Vec<T>
    where
        T: PortableHash,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_slice().portable_hash(state);
        }
    }

    impl PortableHash for alloc::string::String {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_str().portable_hash(state);
        }
    }

    impl PortableHash for alloc::ffi::CString {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_c_str().portable_hash(state);
        }
    }

    impl<T> PortableHash for alloc::borrow::Cow<'_, T>
    where
        T: alloc::borrow::ToOwned + PortableHash + ?Sized,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            T::portable_hash(self, state);
        }
    }

    impl<T> PortableHash for alloc::collections::BTreeSet<T>
    where
        T: PortableHash,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.len().portable_hash(state);
            self.iter().for_each(|item| item.portable_hash(state));
        }
    }

    impl<K, V> PortableHash for alloc::collections::BTreeMap<K, V>
    where
        K: PortableHash,
        V: PortableHash,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.len().portable_hash(state);
            self.iter().for_each(|entry| entry.portable_hash(state));
        }
    }

    impl<T> PortableHash for alloc::collections::LinkedList<T>
    where
        T: PortableHash,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.len().portable_hash(state);
            self.iter().for_each(|item| item.portable_hash(state));
        }
    }

    impl<T> PortableHash for alloc::collections::VecDeque<T>
    where
        T: PortableHash,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.len().portable_hash(state);
            let (first, last) = self.as_slices();
            T::portable_hash_slice(first, state);
            T::portable_hash_slice(last, state);
        }
    }
} _ => {});

cfg_select!(feature = "rend-0_5" => {
    impl PortableHash for rend_0_5::u16_le {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.to_native().portable_hash(state);
        }
    }

    impl PortableHash for rend_0_5::u16_be {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.to_native().portable_hash(state);
        }
    }

    impl PortableHash for rend_0_5::u32_le {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.to_native().portable_hash(state);
        }
    }

    impl PortableHash for rend_0_5::u32_be {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.to_native().portable_hash(state);
        }
    }

    impl PortableHash for rend_0_5::u64_le {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            // SAFETY: `u64_le` is a `repr(C)` newtype over `u64` with equal size and alignment, so it has no padding, so every byte of the value is
            // initialized.
            state.write(unsafe { as_bytes(self) })
        }

        fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
        where
            H: Hasher,
        {
            // SAFETY: `u64_le` is a `repr(C)` newtype over `u64` with equal size and alignment, so it has no padding, so every byte of the slice is
            // initialized.
            state.write(unsafe { slice_as_bytes(slice) });
        }
    }

    impl PortableHash for rend_0_5::u64_be {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.to_native().portable_hash(state);
        }
    }

    impl PortableHash for rend_0_5::u128_le {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            // SAFETY: `u128_le` is a `repr(C)` newtype over `u128` with equal size and alignment, so it has no padding, so every byte of the value is
            // initialized.
            state.write(unsafe { as_bytes(self) });
        }

        fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
        where
            H: Hasher,
        {
            // SAFETY: `u128_le` is a `repr(C)` newtype over `u128` with equal size and alignment, so it has no padding, so every byte of the slice is
            // initialized.
            state.write(unsafe { slice_as_bytes(slice) });
        }
    }

    impl PortableHash for rend_0_5::u128_be {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.to_native().portable_hash(state);
        }
    }

    impl PortableHash for rend_0_5::i16_le {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.to_native().portable_hash(state);
        }
    }

    impl PortableHash for rend_0_5::i16_be {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.to_native().portable_hash(state);
        }
    }

    impl PortableHash for rend_0_5::i32_le {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.to_native().portable_hash(state);
        }
    }

    impl PortableHash for rend_0_5::i32_be {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.to_native().portable_hash(state);
        }
    }

    impl PortableHash for rend_0_5::i64_le {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            // SAFETY: `i64_le` is a `repr(C)` newtype over `i64` with equal size and alignment, so it has no padding, so every byte of the value is
            // initialized.
            state.write(unsafe { as_bytes(self) });
        }

        fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
        where
            H: Hasher,
        {
            // SAFETY: `i64_le` is a `repr(C)` newtype over `i64` with equal size and alignment, so it has no padding, so every byte of the slice is
            // initialized.
            state.write(unsafe { slice_as_bytes(slice) });
        }
    }

    impl PortableHash for rend_0_5::i64_be {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.to_native().portable_hash(state);
        }
    }

    impl PortableHash for rend_0_5::i128_le {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            // SAFETY: `i128_le` is a `repr(C)` newtype over `i128` with equal size and alignment, so it has no padding, so every byte of the value is
            // initialized.
            state.write(unsafe { as_bytes(self) });
        }

        fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
        where
            H: Hasher,
        {
            // SAFETY: `i128_le` is a `repr(C)` newtype over `i128` with equal size and alignment, so it has no padding, so every byte of the slice is
            // initialized.
            state.write(unsafe { slice_as_bytes(slice) });
        }
    }

    impl PortableHash for rend_0_5::i128_be {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.to_native().portable_hash(state);
        }
    }

    impl PortableHash for rend_0_5::char_le {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            // SAFETY: `char_le` is a `repr(C)` newtype over `char` with equal size and alignment, so it has no padding, so every byte of the value is
            // initialized.
            state.write(unsafe { as_bytes(self) });
        }

        fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
        where
            H: Hasher,
        {
            // SAFETY: `char_le` is a `repr(C)` newtype over `char` with equal size and alignment, so it has no padding, so every byte of the slice is
            // initialized.
            state.write(unsafe { slice_as_bytes(slice) });
        }
    }

    impl PortableHash for rend_0_5::char_be {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.to_native().portable_hash(state);
        }
    }

    impl PortableHash for rend_0_5::NonZeroU16_le {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.to_native().portable_hash(state);
        }
    }

    impl PortableHash for rend_0_5::NonZeroU16_be {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.to_native().portable_hash(state);
        }
    }

    impl PortableHash for rend_0_5::NonZeroU32_le {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.to_native().portable_hash(state);
        }
    }

    impl PortableHash for rend_0_5::NonZeroU32_be {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.to_native().portable_hash(state);
        }
    }

    impl PortableHash for rend_0_5::NonZeroU64_le {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            // SAFETY: `NonZeroU64_le` has the same layout as `u64_le`, which has no padding, so every byte of the value is
            // initialized.
            state.write(unsafe { as_bytes(self) });
        }

        fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
        where
            H: Hasher,
        {
            // SAFETY: `NonZeroU64_le` has the same layout as `u64_le`, which has no padding, so every byte of the slice is
            // initialized.
            state.write(unsafe { slice_as_bytes(slice) });
        }
    }

    impl PortableHash for rend_0_5::NonZeroU64_be {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.to_native().portable_hash(state);
        }
    }

    impl PortableHash for rend_0_5::NonZeroU128_le {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            // SAFETY: `NonZeroU128_le` has the same layout as `u128_le`, which has no padding, so every byte of the value is
            // initialized.
            state.write(unsafe { as_bytes(self) });
        }

        fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
        where
            H: Hasher,
        {
            // SAFETY: `NonZeroU128_le` has the same layout as `u128_le`, which has no padding, so every byte of the slice is
            // initialized.
            state.write(unsafe { slice_as_bytes(slice) });
        }
    }

    impl PortableHash for rend_0_5::NonZeroU128_be {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.to_native().portable_hash(state);
        }
    }

    impl PortableHash for rend_0_5::NonZeroI16_le {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.to_native().portable_hash(state);
        }
    }

    impl PortableHash for rend_0_5::NonZeroI16_be {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.to_native().portable_hash(state);
        }
    }

    impl PortableHash for rend_0_5::NonZeroI32_le {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.to_native().portable_hash(state);
        }
    }

    impl PortableHash for rend_0_5::NonZeroI32_be {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.to_native().portable_hash(state);
        }
    }

    impl PortableHash for rend_0_5::NonZeroI64_le {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            // SAFETY: `NonZeroI64_le` has the same layout as `i64_le`, which has no padding, so every byte of the value is
            // initialized.
            state.write(unsafe { as_bytes(self) });
        }

        fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
        where
            H: Hasher,
        {
            // SAFETY: `NonZeroI64_le` has the same layout as `i64_le`, which has no padding, so every byte of the slice is
            // initialized.
            state.write(unsafe { slice_as_bytes(slice) });
        }
    }

    impl PortableHash for rend_0_5::NonZeroI64_be {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.to_native().portable_hash(state);
        }
    }

    impl PortableHash for rend_0_5::NonZeroI128_le {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            // SAFETY: `NonZeroI128_le` has the same layout as `i128_le`, which has no padding, so every byte of the value is
            // initialized.
            state.write(unsafe { as_bytes(self) });
        }

        fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
        where
            H: Hasher,
        {
            // SAFETY: `NonZeroI128_le` has the same layout as `i128_le`, which has no padding, so every byte of the slice is
            // initialized.
            state.write(unsafe { slice_as_bytes(slice) });
        }
    }

    impl PortableHash for rend_0_5::NonZeroI128_be {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.to_native().portable_hash(state);
        }
    }
} _ => {});

cfg_select!(feature = "rkyv-0_8" => {
    impl<T> PortableHash for rkyv_0_8::seal::Seal<'_, T>
    where
        T: PortableHash + ?Sized,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            T::portable_hash(self, state);
        }
    }

    impl<T> PortableHash for rkyv_0_8::boxed::ArchivedBox<T>
    where
        T: rkyv_0_8::traits::ArchivePointee + PortableHash + ?Sized,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            T::portable_hash(self, state)
        }
    }

    #[cfg(feature = "alloc")]
    impl<T, const E: usize> PortableHash for rkyv_0_8::collections::btree_set::ArchivedBTreeSet<T, E>
    where
        T: PortableHash,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.len().portable_hash(state);
            self.iter().for_each(|item| item.portable_hash(state));
        }
    }

    #[cfg(feature = "alloc")]
    impl<K, V, const E: usize> PortableHash for rkyv_0_8::collections::btree_map::ArchivedBTreeMap<K, V, E>
    where
        K: PortableHash,
        V: PortableHash,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.len().portable_hash(state);
            self.iter().for_each(|entry| entry.portable_hash(state));
        }
    }

    impl PortableHash for rkyv_0_8::ffi::ArchivedCString {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_c_str().portable_hash(state);
        }
    }

    impl PortableHash for rkyv_0_8::net::ArchivedIpv4Addr {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            false.portable_hash(state);
            state.write(&self.octets());
        }
    }

    impl PortableHash for rkyv_0_8::net::ArchivedIpv6Addr {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            true.portable_hash(state);
            state.write(&self.octets());
        }
    }

    impl PortableHash for rkyv_0_8::net::ArchivedIpAddr {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            match self {
                Self::V4(ip) => ip.portable_hash(state),
                Self::V6(ip) => ip.portable_hash(state),
            }
        }
    }

    impl PortableHash for rkyv_0_8::net::ArchivedSocketAddrV4 {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            (self.ip(), self.port()).portable_hash(state);
        }
    }

    impl PortableHash for rkyv_0_8::net::ArchivedSocketAddrV6 {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            (self.ip(), self.port()).portable_hash(state);
            // Both fields sit at a fixed position in a fixed-width encoding, so they are
            // written at their own width rather than widened the way a bare `u32` would be.
            state.write(&self.flowinfo().to_le_bytes());
            state.write(&self.scope_id().to_le_bytes());
        }
    }

    impl PortableHash for rkyv_0_8::net::ArchivedSocketAddr {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            match self {
                Self::V4(addr) => addr.portable_hash(state),
                Self::V6(addr) => addr.portable_hash(state),
            }
        }
    }

    impl<T, N> PortableHash for rkyv_0_8::niche::niched_option::NichedOption<T, N>
    where
        T: PortableHash,
        N: rkyv_0_8::niche::niching::Niching<T> + ?Sized,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_ref().portable_hash(state);
        }
    }

    impl<T> PortableHash for rkyv_0_8::niche::option_box::ArchivedOptionBox<T>
    where
        T: PortableHash + rkyv_0_8::traits::ArchivePointee + ?Sized,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_ref().portable_hash(state);
        }
    }

    impl PortableHash for rkyv_0_8::niche::option_nonzero::ArchivedOptionNonZeroU8 {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_ref().portable_hash(state)
        }
    }

    impl PortableHash for rkyv_0_8::niche::option_nonzero::ArchivedOptionNonZeroU16 {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_ref().portable_hash(state)
        }
    }

    impl PortableHash for rkyv_0_8::niche::option_nonzero::ArchivedOptionNonZeroU32 {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_ref().portable_hash(state)
        }
    }

    impl PortableHash for rkyv_0_8::niche::option_nonzero::ArchivedOptionNonZeroU64 {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_ref().portable_hash(state)
        }
    }

    impl PortableHash for rkyv_0_8::niche::option_nonzero::ArchivedOptionNonZeroU128 {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_ref().portable_hash(state)
        }
    }

    impl PortableHash for rkyv_0_8::niche::option_nonzero::ArchivedOptionNonZeroI8 {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_ref().portable_hash(state)
        }
    }

    impl PortableHash for rkyv_0_8::niche::option_nonzero::ArchivedOptionNonZeroI16 {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_ref().portable_hash(state)
        }
    }

    impl PortableHash for rkyv_0_8::niche::option_nonzero::ArchivedOptionNonZeroI32 {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_ref().portable_hash(state)
        }
    }

    impl PortableHash for rkyv_0_8::niche::option_nonzero::ArchivedOptionNonZeroI64 {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_ref().portable_hash(state)
        }
    }

    impl PortableHash for rkyv_0_8::niche::option_nonzero::ArchivedOptionNonZeroI128 {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_ref().portable_hash(state)
        }
    }

    impl<T> PortableHash for rkyv_0_8::ops::ArchivedBound<T>
    where
        T: PortableHash,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_ref().portable_hash(state);
        }
    }

    impl<T> PortableHash for rkyv_0_8::ops::ArchivedRange<T>
    where
        T: PortableHash,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            (self.start_bound(), self.end_bound()).portable_hash(state);
        }
    }

    impl<T> PortableHash for rkyv_0_8::ops::ArchivedRangeInclusive<T>
    where
        T: PortableHash,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            (self.start_bound(), self.end_bound()).portable_hash(state);
        }
    }

    impl<T> PortableHash for rkyv_0_8::ops::ArchivedRangeFrom<T>
    where
        T: PortableHash,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            (self.start_bound(), self.end_bound()).portable_hash(state);
        }
    }

    impl<T> PortableHash for rkyv_0_8::ops::ArchivedRangeTo<T>
    where
        T: PortableHash,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            (self.start_bound(), self.end_bound()).portable_hash(state);
        }
    }

    impl<T> PortableHash for rkyv_0_8::ops::ArchivedRangeToInclusive<T>
    where
        T: PortableHash,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            (self.start_bound(), self.end_bound()).portable_hash(state);
        }
    }

    impl PortableHash for rkyv_0_8::ops::ArchivedRangeFull {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            (..).portable_hash(state);
        }
    }

    impl<T> PortableHash for rkyv_0_8::option::ArchivedOption<T>
    where
        T: PortableHash,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_ref().portable_hash(state);
        }
    }

    impl<T, F> PortableHash for rkyv_0_8::rc::ArchivedRc<T, F>
    where
        T: PortableHash + rkyv_0_8::traits::ArchivePointee + ?Sized,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            T::portable_hash(self, state);
        }
    }

    impl<T, E> PortableHash for rkyv_0_8::result::ArchivedResult<T, E>
    where
        T: PortableHash,
        E: PortableHash,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_ref().portable_hash(state);
        }
    }

    impl PortableHash for rkyv_0_8::string::ArchivedString {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_str().portable_hash(state);
        }
    }

    impl PortableHash for rkyv_0_8::time::ArchivedDuration {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_secs().portable_hash(state);
            self.subsec_nanos().portable_hash(state);
        }
    }

    impl<T> PortableHash for rkyv_0_8::util::Align<T>
    where
        T: PortableHash,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.0.portable_hash(state);
        }
    }

    #[cfg(feature = "alloc")]
    impl<const ALIGNMENT: usize> PortableHash for rkyv_0_8::util::AlignedVec<ALIGNMENT> {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_slice().portable_hash(state);
        }
    }

    impl<T, const N: usize> PortableHash for rkyv_0_8::util::InlineVec<T, N>
    where
        T: PortableHash,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_slice().portable_hash(state);
        }
    }

    impl<T> PortableHash for rkyv_0_8::util::SerVec<T>
    where
        T: PortableHash,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_slice().portable_hash(state);
        }
    }

    impl<T> PortableHash for rkyv_0_8::vec::ArchivedVec<T>
    where
        T: PortableHash,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_slice().portable_hash(state);
        }
    }
} _ => {});

cfg_select!(feature = "allocator-api2-0_4" => {
    impl<T, A> PortableHash for allocator_api2_0_4::boxed::Box<T, A>
    where
        T: PortableHash + ?Sized,
        A: allocator_api2_0_4::alloc::Allocator,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            T::portable_hash(self, state);
        }
    }

    impl<T, A> PortableHash for allocator_api2_0_4::vec::Vec<T, A>
    where
        T: PortableHash,
        A: allocator_api2_0_4::alloc::Allocator,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_slice().portable_hash(state);
        }
    }
} _ => {});

cfg_select!(feature = "allocator-api2-0_3" => {
    impl<T, A> PortableHash for allocator_api2_0_3::boxed::Box<T, A>
    where
        T: PortableHash + ?Sized,
        A: allocator_api2_0_3::alloc::Allocator,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            T::portable_hash(self, state);
        }
    }

    impl<T, A> PortableHash for allocator_api2_0_3::vec::Vec<T, A>
    where
        T: PortableHash,
        A: allocator_api2_0_3::alloc::Allocator,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_slice().portable_hash(state);
        }
    }
} _ => {});

cfg_select!(feature = "allocator-api2-0_2" => {
    impl<T, A> PortableHash for allocator_api2_0_2::boxed::Box<T, A>
    where
        T: PortableHash + ?Sized,
        A: allocator_api2_0_2::alloc::Allocator,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            T::portable_hash(self, state);
        }
    }

    impl<T, A> PortableHash for allocator_api2_0_2::vec::Vec<T, A>
    where
        T: PortableHash,
        A: allocator_api2_0_2::alloc::Allocator,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_slice().portable_hash(state);
        }
    }
} _ => {});

cfg_select!(feature = "arrayvec-0_7" => {
    impl<T, const CAP: usize> PortableHash for arrayvec_0_7::ArrayVec<T, CAP>
    where
        T: PortableHash,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_slice().portable_hash(state);
        }
    }

    impl<const CAP: usize> PortableHash for arrayvec_0_7::ArrayString<CAP> {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_str().portable_hash(state);
        }
    }
} _ => {});

cfg_select!(feature = "ascii-1" => {
    impl PortableHash for ascii_1::AsciiChar {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_char().portable_hash(state);
        }
    }

    impl PortableHash for ascii_1::AsciiStr {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_str().portable_hash(state);
        }
    }

    #[cfg(feature = "alloc")]
    impl PortableHash for ascii_1::AsciiString {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_str().portable_hash(state);
        }
    }
} _ => {});

cfg_select!(feature = "bstr-1" => {
    impl PortableHash for bstr_1::BStr {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.len().portable_hash(state);
            state.write(self);
        }
    }

    #[cfg(feature = "alloc")]
    impl PortableHash for bstr_1::BString {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.len().portable_hash(state);
            state.write(self);
        }
    }
} _ => {});

cfg_select!(feature = "bumpalo-3" => {
    impl<T> PortableHash for bumpalo_3::boxed::Box<'_, T>
    where
        T: PortableHash + ?Sized,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            T::portable_hash(self, state);
        }
    }

    impl<T> PortableHash for bumpalo_3::collections::Vec<'_, T>
    where
        T: PortableHash,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_slice().portable_hash(state);
        }
    }

    impl PortableHash for bumpalo_3::collections::String<'_> {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_str().portable_hash(state);
        }
    }
} _ => {});

cfg_select!(feature = "bytes-1" => {
    impl PortableHash for bytes_1::Bytes {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.len().portable_hash(state);
            state.write(self);
        }
    }

    impl PortableHash for bytes_1::BytesMut {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.len().portable_hash(state);
            state.write(self);
        }
    }
} _ => {});

cfg_select!(feature = "smallvec-1" => {
    impl<A> PortableHash for smallvec_1::SmallVec<A>
    where
        A: smallvec_1::Array,
        A::Item: PortableHash,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_slice().portable_hash(state);
        }
    }
} _ => {});

cfg_select!(feature = "smol_str-0_2" => {
    impl PortableHash for smol_str_0_2::SmolStr {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_str().portable_hash(state);
        }
    }
} _ => {});

cfg_select!(feature = "smol_str-0_3" => {
    impl PortableHash for smol_str_0_3::SmolStr {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_str().portable_hash(state);
        }
    }
} _ => {});

cfg_select!(feature = "thin-vec-0_2" => {
    impl<T> PortableHash for thin_vec_0_2::ThinVec<T>
    where
        T: PortableHash,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_slice().portable_hash(state);
        }
    }
} _ => {});

cfg_select!(feature = "tinyvec-1" => {
    impl<A> PortableHash for tinyvec_1::ArrayVec<A>
    where
        A: tinyvec_1::Array,
        A::Item: PortableHash,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_slice().portable_hash(state);
        }
    }

    impl<T> PortableHash for tinyvec_1::SliceVec<'_, T>
    where
        T: PortableHash,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_slice().portable_hash(state);
        }
    }

    #[cfg(feature = "alloc")]
    impl<A> PortableHash for tinyvec_1::TinyVec<A>
    where
        A: tinyvec_1::Array,
        A::Item: PortableHash,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_slice().portable_hash(state);
        }
    }
} _ => {});

cfg_select!(feature = "triomphe-0_1" => {
    impl<T> PortableHash for triomphe_0_1::Arc<T>
    where
        T: PortableHash + ?Sized,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            T::portable_hash(self, state);
        }
    }

    impl<T> PortableHash for triomphe_0_1::ArcBorrow<'_, T>
    where
        T: PortableHash,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            T::portable_hash(self.get(), state);
        }
    }

    impl<A, B> PortableHash for triomphe_0_1::ArcUnionBorrow<'_, A, B>
    where
        A: PortableHash,
        B: PortableHash,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            match self {
                Self::First(first) => (false, first).portable_hash(state),
                Self::Second(second) => (true, second).portable_hash(state),
            }
        }
    }

    impl<A, B> PortableHash for triomphe_0_1::ArcUnion<A, B>
    where
        A: PortableHash,
        B: PortableHash,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.borrow().portable_hash(state);
        }
    }

    impl<H, T> PortableHash for triomphe_0_1::ThinArc<H, T>
    where
        H: PortableHash,
        T: PortableHash,
    {
        fn portable_hash<S>(&self, state: &mut S)
        where
            S: Hasher,
        {
            self.header.header.portable_hash(state);
            self.slice.portable_hash(state);
        }
    }

    impl<T> PortableHash for triomphe_0_1::OffsetArc<T>
    where
        T: PortableHash + ?Sized,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            T::portable_hash(self, state);
        }
    }

    impl<T> PortableHash for triomphe_0_1::UniqueArc<T>
    where
        T: PortableHash + ?Sized,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            T::portable_hash(self, state);
        }
    }
} _ => {});

cfg_select!(feature = "either-1" => {
    impl<L, R> PortableHash for either_1::Either<L, R>
    where
        L: PortableHash,
        R: PortableHash,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            match self {
                Self::Left(l) => (false, l).portable_hash(state),
                Self::Right(r) => (true, r).portable_hash(state),
            }
        }
    }
} _ => {});

#[cfg(test)]
mod tests {
    use super::PortableHash;

    use crate::eq::PortableEq;
    use crate::{AssertPortable, Portable};

    use core::num::{
        NonZeroI8, NonZeroI64, NonZeroI128, NonZeroIsize, NonZeroU8, NonZeroU64, NonZeroU128,
        NonZeroUsize, Saturating, Wrapping,
    };

    use std::vec::Vec;

    /// Hasher that records the bytes written to it, so hashes can be compared byte for byte.
    #[derive(Default)]
    struct Recorder(Vec<u8>);

    impl core::hash::Hasher for Recorder {
        fn finish(&self) -> u64 {
            0
        }

        fn write(&mut self, bytes: &[u8]) {
            self.0.extend_from_slice(bytes);
        }
    }

    fn bytes_of<T>(value: &T) -> Vec<u8>
    where
        T: PortableHash + ?Sized,
    {
        let mut state = Recorder::default();
        value.portable_hash(&mut state);
        state.0
    }

    /// Asserts that `T`'s `portable_hash_slice` writes exactly what hashing each element
    /// separately would, which is the only thing overriding it is allowed to change.
    fn assert_slice_shortcut_matches_elements<T>(values: &[T])
    where
        T: PortableHash,
    {
        for len in 0..=values.len() {
            let slice = &values[..len];

            let mut shortcut = Recorder::default();
            T::portable_hash_slice(slice, &mut shortcut);

            let mut element_wise = Recorder::default();
            slice
                .iter()
                .for_each(|value| value.portable_hash(&mut element_wise));

            assert_eq!(shortcut.0, element_wise.0, "length {len}");
        }
    }

    #[test]
    fn integers_are_hashed_as_little_endian_bytes_of_their_widest_form() {
        // Every integer that a `usize` may be stored as widens to the same eight bytes, so
        // values that compare equal across widths hash equally.
        let one = 1u64.to_le_bytes().to_vec();

        assert_eq!(bytes_of(&1u16), one);
        assert_eq!(bytes_of(&1u32), one);
        assert_eq!(bytes_of(&1u64), one);
        assert_eq!(bytes_of(&1usize), one);

        let minus_one = (-1i64).to_le_bytes().to_vec();

        assert_eq!(bytes_of(&-1i16), minus_one);
        assert_eq!(bytes_of(&-1i32), minus_one);
        assert_eq!(bytes_of(&-1i64), minus_one);
        assert_eq!(bytes_of(&-1isize), minus_one);

        assert_eq!(bytes_of(&1u128), 1u128.to_le_bytes().to_vec());
        assert_eq!(bytes_of(&-1i128), (-1i128).to_le_bytes().to_vec());

        // Byte-wide values are written as a single byte.
        assert_eq!(bytes_of(&1u8), std::vec![1]);
        assert_eq!(bytes_of(&-1i8), std::vec![0xff]);
        assert_eq!(bytes_of(&true), std::vec![1]);
        assert_eq!(bytes_of(&false), std::vec![0]);
    }

    #[test]
    fn other_primitives_are_hashed_as_fixed_little_endian_byte_sequences() {
        assert_eq!(bytes_of(&'A'), 65u32.to_le_bytes().to_vec());
        assert_eq!(bytes_of(&'\u{10ffff}'), 0x10ffffu32.to_le_bytes().to_vec());
        assert_eq!(bytes_of(&()), Vec::<u8>::new());
        assert_eq!(
            bytes_of(&core::marker::PhantomData::<u32>),
            Vec::<u8>::new()
        );

        assert_eq!(bytes_of(&core::cmp::Ordering::Less), std::vec![0xff]);
        assert_eq!(bytes_of(&core::cmp::Ordering::Equal), std::vec![0]);
        assert_eq!(bytes_of(&core::cmp::Ordering::Greater), std::vec![1]);
    }

    #[test]
    fn strings_are_hashed_so_that_no_encoding_is_a_prefix_of_another() {
        // The terminator keeps concatenations from colliding.
        assert_ne!(
            [bytes_of("a"), bytes_of("b")].concat(),
            [bytes_of("ab"), bytes_of("")].concat(),
        );
        assert_eq!(bytes_of("hi"), [b"hi".as_slice(), b"\xff"].concat());
        assert_eq!(bytes_of(c"hi"), b"hi\0".to_vec());
    }

    #[test]
    fn slices_hash_their_length() {
        // Without a length the nesting would be ambiguous.
        assert_ne!(
            bytes_of(&[&[1u32][..], &[2, 3][..]][..]),
            bytes_of(&[&[1u32, 2][..], &[3][..]][..]),
        );
        assert_ne!(bytes_of(&[1u32, 2][..]), bytes_of(&[1u32, 2, 0][..]));
    }

    #[test]
    fn slice_shortcuts_agree_with_hashing_elements_separately() {
        assert_slice_shortcut_matches_elements(&[true, false, true]);
        assert_slice_shortcut_matches_elements(&[1u8, 2, 0xff]);
        assert_slice_shortcut_matches_elements(&[1u64, 2, u64::MAX]);
        assert_slice_shortcut_matches_elements(&[1u128, 2, u128::MAX]);
        assert_slice_shortcut_matches_elements(&[1usize, 2, usize::MAX]);
        assert_slice_shortcut_matches_elements(&[-1i8, 2, i8::MIN]);
        assert_slice_shortcut_matches_elements(&[-1i64, 2, i64::MIN]);
        assert_slice_shortcut_matches_elements(&[-1i128, 2, i128::MIN]);
        assert_slice_shortcut_matches_elements(&[-1isize, 2, isize::MIN]);
        assert_slice_shortcut_matches_elements(&['a', '\u{10ffff}', '\0']);

        assert_slice_shortcut_matches_elements(&[
            NonZeroU8::new(1).unwrap(),
            NonZeroU8::new(2).unwrap(),
        ]);
        assert_slice_shortcut_matches_elements(&[
            NonZeroU64::new(1).unwrap(),
            NonZeroU64::new(2).unwrap(),
        ]);
        assert_slice_shortcut_matches_elements(&[
            NonZeroU128::new(1).unwrap(),
            NonZeroU128::new(2).unwrap(),
        ]);
        assert_slice_shortcut_matches_elements(&[
            NonZeroUsize::new(1).unwrap(),
            NonZeroUsize::new(2).unwrap(),
        ]);
        assert_slice_shortcut_matches_elements(&[
            NonZeroI8::new(-1).unwrap(),
            NonZeroI8::new(2).unwrap(),
        ]);
        assert_slice_shortcut_matches_elements(&[
            NonZeroI64::new(-1).unwrap(),
            NonZeroI64::new(2).unwrap(),
        ]);
        assert_slice_shortcut_matches_elements(&[
            NonZeroI128::new(-1).unwrap(),
            NonZeroI128::new(2).unwrap(),
        ]);
        assert_slice_shortcut_matches_elements(&[
            NonZeroIsize::new(-1).unwrap(),
            NonZeroIsize::new(2).unwrap(),
        ]);

        assert_slice_shortcut_matches_elements(&[(), ()]);
        assert_slice_shortcut_matches_elements(&[core::marker::PhantomData::<u32>; 2]);
        assert_slice_shortcut_matches_elements(&[
            core::marker::PhantomPinned,
            core::marker::PhantomPinned,
        ]);
        assert_slice_shortcut_matches_elements(&[.., ..]);

        // Transparent wrappers reinterpret the slice rather than walking it.
        assert_slice_shortcut_matches_elements(&[core::cmp::Reverse(1u64), core::cmp::Reverse(2)]);
        assert_slice_shortcut_matches_elements(&[Saturating(1u64), Saturating(2)]);
        assert_slice_shortcut_matches_elements(&[Wrapping(1u64), Wrapping(2)]);
        assert_slice_shortcut_matches_elements(&[Portable(1u64), Portable(2)]);
        assert_slice_shortcut_matches_elements(&[Portable(1u16), Portable(2)]);
    }

    #[cfg(feature = "rend-0_5")]
    #[test]
    fn endian_aware_slice_shortcuts_agree_with_hashing_elements_separately() {
        use rend_0_5::{
            NonZeroI64_le, NonZeroI128_le, NonZeroU64_le, NonZeroU128_le, char_le, i64_le, i128_le,
            u64_le, u128_le,
        };

        assert_slice_shortcut_matches_elements(&[
            u64_le::from_native(1),
            u64_le::from_native(u64::MAX),
        ]);
        assert_slice_shortcut_matches_elements(&[
            u128_le::from_native(1),
            u128_le::from_native(u128::MAX),
        ]);
        assert_slice_shortcut_matches_elements(&[
            i64_le::from_native(-1),
            i64_le::from_native(i64::MIN),
        ]);
        assert_slice_shortcut_matches_elements(&[
            i128_le::from_native(-1),
            i128_le::from_native(i128::MIN),
        ]);
        assert_slice_shortcut_matches_elements(&[
            char_le::from_native('a'),
            char_le::from_native('\u{10ffff}'),
        ]);
        assert_slice_shortcut_matches_elements(&[
            NonZeroU64_le::from_native(NonZeroU64::new(1).unwrap()),
            NonZeroU64_le::from_native(NonZeroU64::new(2).unwrap()),
        ]);
        assert_slice_shortcut_matches_elements(&[
            NonZeroU128_le::from_native(NonZeroU128::new(1).unwrap()),
            NonZeroU128_le::from_native(NonZeroU128::new(2).unwrap()),
        ]);
        assert_slice_shortcut_matches_elements(&[
            NonZeroI64_le::from_native(NonZeroI64::new(-1).unwrap()),
            NonZeroI64_le::from_native(NonZeroI64::new(2).unwrap()),
        ]);
        assert_slice_shortcut_matches_elements(&[
            NonZeroI128_le::from_native(NonZeroI128::new(-1).unwrap()),
            NonZeroI128_le::from_native(NonZeroI128::new(2).unwrap()),
        ]);
    }

    #[test]
    fn enum_variants_are_distinguished_by_a_discriminant() {
        assert_ne!(bytes_of(&Some(0u32)), bytes_of(&None::<u32>));
        assert_ne!(bytes_of(&Ok::<u32, u32>(0)), bytes_of(&Err::<u32, u32>(0)));
        assert_ne!(
            bytes_of(&core::task::Poll::Ready(0u32)),
            bytes_of(&core::task::Poll::<u32>::Pending),
        );

        use core::ops::Bound::{Excluded, Included, Unbounded};

        assert_ne!(bytes_of(&Included(0u32)), bytes_of(&Excluded(0u32)));
        assert_ne!(bytes_of(&Included(0u32)), bytes_of(&Unbounded::<u32>));
    }

    #[test]
    fn ranges_of_every_kind_hash_their_bounds() {
        assert_eq!(bytes_of(&(1u32..3)), bytes_of(&(1u32..3)));
        assert_ne!(bytes_of(&(1u32..3)), bytes_of(&(1u32..=3)));
        assert_ne!(bytes_of(&(1u32..)), bytes_of(&(..1u32)));
        assert_ne!(bytes_of(&(1u32..3)), bytes_of(&(..)));
    }

    /// Asserts the contract between [`PortableEq`] and [`PortableHash`]: values that compare
    /// equal hash equally, even when their types differ.
    macro_rules! assert_contract {
        ($left:expr, $right:expr $(,)?) => {{
            let (left, right) = ($left, $right);

            assert!(left.portable_eq(right), "expected equality");
            assert_eq!(bytes_of(left), bytes_of(right), "expected equal hashes");
        }};
    }

    #[test]
    fn equal_values_hash_equally_across_types() {
        assert_contract!(&1usize, &1u16);
        assert_contract!(&1usize, &1u32);
        assert_contract!(&1usize, &1u64);
        assert_contract!(&-1isize, &-1i16);
        assert_contract!(&-1isize, &-1i32);
        assert_contract!(&-1isize, &-1i64);

        assert_contract!(&NonZeroU64::new(1).unwrap(), &1u64);
        assert_contract!(&NonZeroUsize::new(1).unwrap(), &1usize);
        assert_contract!(&Wrapping(1u32), &1u32);
        assert_contract!(&Saturating(1u32), &1u32);
        assert_contract!(&core::panic::AssertUnwindSafe(1u32), &1u32);

        assert_contract!(&[1u32, 2, 3], &[1u32, 2, 3][..]);
        assert_contract!(&Some(1usize), &Some(1u64));
        assert_contract!(&Ok::<usize, u8>(1), &Ok::<u64, u8>(1));
        assert_contract!(&(1usize, 'x'), &(1u64, 'x'));

        // A `SocketAddr` and its variant type share a representation.
        let v4: core::net::SocketAddrV4 = "127.0.0.1:80".parse().unwrap();
        assert_contract!(&v4, &core::net::SocketAddr::V4(v4));

        // `flowinfo` and `scope_id` are part of a `SocketAddrV6`'s identity, so they must be
        // carried across the representation too.
        let v6 = core::net::SocketAddrV6::new(core::net::Ipv6Addr::LOCALHOST, 80, 7, 9);
        assert_contract!(&v6, &core::net::SocketAddr::V6(v6));

        // So do an `IpAddr` and the address it holds.
        assert_contract!(
            &core::net::Ipv4Addr::LOCALHOST,
            &core::net::IpAddr::V4(core::net::Ipv4Addr::LOCALHOST)
        );
        assert_contract!(
            &core::net::Ipv6Addr::LOCALHOST,
            &core::net::IpAddr::V6(core::net::Ipv6Addr::LOCALHOST)
        );
    }

    #[test]
    fn socket_addresses_hash_every_field_their_equality_compares() {
        use core::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

        let v6 = SocketAddrV6::new(Ipv6Addr::LOCALHOST, 80, 7, 9);

        for other in [
            SocketAddrV6::new(Ipv6Addr::LOCALHOST, 80, 0, 9),
            SocketAddrV6::new(Ipv6Addr::LOCALHOST, 80, 7, 0),
            SocketAddrV6::new(Ipv6Addr::LOCALHOST, 81, 7, 9),
        ] {
            assert!(!v6.portable_eq(&other));
            assert_ne!(bytes_of(&v6), bytes_of(&other));
        }

        // The two families are tagged by the address they hold, so neither can collide with
        // the other and neither needs a tag of its own.
        let v4 = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 80);

        assert_ne!(bytes_of(&SocketAddr::V4(v4)), bytes_of(&SocketAddr::V6(v6)));
        assert_ne!(
            bytes_of(&Ipv4Addr::LOCALHOST),
            bytes_of(&Ipv6Addr::LOCALHOST)
        );

        // Every field is fixed width, so the encoding has a fixed length: a tagged address, a
        // widened port, then `flowinfo` and `scope_id` at their own four bytes each.
        assert_eq!(bytes_of(&v4).len(), 1 + 4 + 8);
        assert_eq!(bytes_of(&v6).len(), 1 + 16 + 8 + 4 + 4);
        assert_eq!(bytes_of(&v6)[25..], [7, 0, 0, 0, 9, 0, 0, 0]);
    }

    #[cfg(feature = "rend-0_5")]
    #[test]
    fn endian_aware_values_hash_like_their_native_form() {
        assert_contract!(&rend_0_5::u32_le::from_native(5), &5u32);
        assert_contract!(&rend_0_5::u32_be::from_native(5), &5u32);
        assert_contract!(&rend_0_5::u64_le::from_native(5), &5u64);
        assert_contract!(&rend_0_5::u64_be::from_native(5), &5u64);
        assert_contract!(&rend_0_5::u64_le::from_native(5), &5usize);
        assert_contract!(&rend_0_5::u128_le::from_native(5), &5u128);
        assert_contract!(&rend_0_5::i64_le::from_native(-5), &-5i64);
        assert_contract!(&rend_0_5::i128_le::from_native(-5), &-5i128);
        assert_contract!(&rend_0_5::char_le::from_native('x'), &'x');
        assert_contract!(&rend_0_5::char_be::from_native('x'), &'x');
        assert_contract!(
            &rend_0_5::NonZeroU64_le::from_native(NonZeroU64::new(5).unwrap()),
            &NonZeroU64::new(5).unwrap(),
        );
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn owned_values_hash_like_the_borrowed_forms_they_compare_with() {
        use alloc::borrow::Cow;
        use alloc::boxed::Box;
        use alloc::collections::{BTreeMap, BTreeSet, LinkedList, VecDeque};
        use alloc::string::String;
        use alloc::vec;

        assert_contract!(&vec![1u32, 2, 3], &[1u32, 2, 3][..]);
        assert_contract!(&VecDeque::from(vec![1u32, 2, 3]), &[1u32, 2, 3][..]);
        assert_contract!(&BTreeSet::from([1u32, 2, 3]), &[1u32, 2, 3][..]);
        assert_contract!(&LinkedList::from([1u32, 2, 3]), &[1u32, 2, 3][..]);
        assert_contract!(
            &BTreeMap::from([(1u32, 10u64), (2, 20)]),
            &[(1u32, 10u64), (2, 20)][..],
        );

        assert_contract!(&String::from("hi"), "hi");
        assert_contract!(&Cow::Borrowed("hi"), "hi");
        assert_contract!(&Box::new(1usize), &1u64);

        // A wrapped-around `VecDeque` hashes its two halves as one logical sequence.
        let mut wrapped = VecDeque::with_capacity(4);
        wrapped.push_back(3u32);
        wrapped.push_front(2);
        wrapped.push_front(1);
        assert_contract!(&wrapped, &[1u32, 2, 3][..]);
    }

    #[cfg(all(feature = "rkyv-0_8", feature = "alloc"))]
    #[test]
    fn archived_values_hash_like_the_values_they_were_archived_from() {
        use alloc::collections::{BTreeMap, BTreeSet};
        use alloc::string::{String, ToString};
        use alloc::vec;

        use rkyv_0_8::rancor::Error;

        macro_rules! assert_archived_contract {
            ($archived:ty, $value:expr) => {{
                let value = $value;
                let bytes = rkyv_0_8::to_bytes::<Error>(&value).unwrap();
                // SAFETY: the bytes were just produced by `to_bytes` for this exact type.
                let archived = unsafe { rkyv_0_8::access_unchecked::<$archived>(&bytes) };

                assert_contract!(archived, &value);
            }};
        }

        assert_archived_contract!(rkyv_0_8::rend::u32_le, 7u32);
        assert_archived_contract!(rkyv_0_8::string::ArchivedString, String::from("hello"));
        assert_archived_contract!(
            rkyv_0_8::vec::ArchivedVec<rkyv_0_8::rend::u32_le>,
            vec![1u32, 2, 3]
        );
        assert_archived_contract!(
            rkyv_0_8::option::ArchivedOption<rkyv_0_8::rend::u32_le>,
            Some(7u32)
        );
        assert_archived_contract!(
            rkyv_0_8::time::ArchivedDuration,
            core::time::Duration::new(3, 4)
        );
        assert_archived_contract!(
            rkyv_0_8::net::ArchivedIpv4Addr,
            core::net::Ipv4Addr::LOCALHOST
        );
        assert_archived_contract!(
            rkyv_0_8::net::ArchivedSocketAddr,
            "127.0.0.1:80".parse::<core::net::SocketAddr>().unwrap()
        );
        assert_archived_contract!(
            rkyv_0_8::net::ArchivedSocketAddr,
            core::net::SocketAddr::V6(core::net::SocketAddrV6::new(
                core::net::Ipv6Addr::LOCALHOST,
                80,
                7,
                9,
            ))
        );
        assert_archived_contract!(
            rkyv_0_8::net::ArchivedSocketAddrV6,
            core::net::SocketAddrV6::new(core::net::Ipv6Addr::LOCALHOST, 80, 7, 9)
        );
        assert_archived_contract!(
            rkyv_0_8::collections::btree_set::ArchivedBTreeSet<rkyv_0_8::rend::u32_le>,
            BTreeSet::from([1u32, 2, 3])
        );
        assert_archived_contract!(
            rkyv_0_8::collections::btree_map::ArchivedBTreeMap<
                rkyv_0_8::rend::u32_le,
                rkyv_0_8::rend::u64_le,
            >,
            BTreeMap::from([(1u32, 10u64), (2, 20)])
        );
        assert_archived_contract!(
            rkyv_0_8::tuple::ArchivedTuple2<rkyv_0_8::rend::u32_le, rkyv_0_8::string::ArchivedString>,
            (1u32, "x".to_string())
        );
        assert_archived_contract!(
            rkyv_0_8::result::ArchivedResult<
                rkyv_0_8::rend::u32_le,
                rkyv_0_8::string::ArchivedString,
            >,
            Err::<u32, String>("bad".to_string())
        );
    }

    #[cfg(feature = "ascii-1")]
    #[test]
    fn ascii_values_hash_like_the_text_they_compare_with() {
        assert_contract!(&ascii_1::AsciiChar::A, &'A');
        assert_contract!(ascii_1::AsciiStr::from_ascii("hi").unwrap(), "hi");

        #[cfg(feature = "alloc")]
        assert_contract!(&ascii_1::AsciiString::from_ascii("hi").unwrap(), "hi");
    }

    #[cfg(feature = "bytes-1")]
    #[test]
    fn byte_buffers_hash_like_the_slices_they_compare_with() {
        assert_contract!(&bytes_1::Bytes::from_static(b"abc"), &b"abc"[..]);
        assert_contract!(&bytes_1::BytesMut::from(&b"abc"[..]), &b"abc"[..]);
    }

    #[cfg(feature = "bstr-1")]
    #[test]
    fn byte_strings_hash_like_the_slices_they_compare_with() {
        assert_contract!(bstr_1::BStr::new(b"abc"), &b"abc"[..]);
    }

    #[test]
    fn wrappers_hash_their_inner_value() {
        assert_eq!(bytes_of(&Portable(1u32)), bytes_of(&1u32));
        assert_eq!(
            bytes_of(&AssertPortable(1u32)),
            bytes_of(&AssertPortable(1u32))
        );
        assert_eq!(bytes_of(&&1u32), bytes_of(&1u32));

        // The standard `Hash` impl on `Portable` must route through `PortableHash`.
        let mut via_std = Recorder::default();
        core::hash::Hash::hash(&Portable(1u32), &mut via_std);

        assert_eq!(via_std.0, bytes_of(&1u32));
    }
}
