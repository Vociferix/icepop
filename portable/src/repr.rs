//! Portable representations: the canonical forms that comparisons are performed on.
//!
//! A type's representation is the form it shares with every other type that should compare as
//! the same value. `u32`, a little-endian `u32` and `NonZeroU32` are all represented as `u32`;
//! `[T; N]`, `Vec<T>` and `Box<[T]>` are all represented as `[T]`. Types that cannot hand out
//! a plain reference to their representation — `Option`, `Result`, tuples, ranges — are
//! represented by the borrowed view types defined here. Collections that are neither a slice
//! nor viewable as one, such as `BTreeMap` and `VecDeque`, represent themselves and compare
//! element-wise against slices and against collections of their own kind.

use super::Portable;

use core::ptr::NonNull;

/// Names a type's portable representation and lends out a view of it.
///
/// Implementing this trait is all a type needs in order to gain
/// [`PortableEq`](crate::PortableEq) and [`PortableOrd`](crate::PortableOrd), and to compare
/// against every other type sharing its representation. The representation is passed to a
/// callback rather than returned, so it may be a temporary — the `u32` decoded from a
/// little-endian `u32`, or a view type such as [`OptionRepr`].
///
/// # Example
///
/// ```
/// use portable::repr::VisitPortableRepr;
///
/// // An array is represented as a slice.
/// let len = [1u32, 2, 3].visit_portable_repr(|repr| repr.len());
///
/// assert_eq!(len, 3);
/// ```
pub trait VisitPortableRepr {
    /// The canonical form this type is compared as.
    type Repr: PortableRepr + ?Sized;

    /// Calls `f` with a view of this value's portable representation.
    ///
    /// # Example
    ///
    /// ```
    /// use portable::repr::VisitPortableRepr;
    ///
    /// // A `NonZeroU32` is represented as a `u32`.
    /// let doubled = core::num::NonZeroU32::new(21)
    ///     .unwrap()
    ///     .visit_portable_repr(|repr| repr * 2);
    ///
    /// assert_eq!(doubled, 42);
    /// ```
    fn visit_portable_repr<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Self::Repr) -> R;
}

/// A type that is its own portable representation.
///
/// Blanket-implemented for every [`VisitPortableRepr`] type whose
/// [`Repr`](VisitPortableRepr::Repr) is itself, so it is never implemented manually. Bounds on
/// representations use it to require that the representation bottoms out.
pub trait PortableRepr: VisitPortableRepr<Repr = Self> {}

impl<T> PortableRepr for T where T: VisitPortableRepr<Repr = Self> + ?Sized {}

impl<T> VisitPortableRepr for Portable<T>
where
    T: VisitPortableRepr + ?Sized,
{
    type Repr = T::Repr;

    fn visit_portable_repr<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Self::Repr) -> R,
    {
        self.0.visit_portable_repr(f)
    }
}

impl<T: ?Sized> VisitPortableRepr for crate::AssertPortable<T> {
    type Repr = Self;

    fn visit_portable_repr<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Self::Repr) -> R,
    {
        f(self)
    }
}

impl<T> VisitPortableRepr for &T
where
    T: VisitPortableRepr + ?Sized,
{
    type Repr = T::Repr;

    fn visit_portable_repr<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Self::Repr) -> R,
    {
        T::visit_portable_repr(self, f)
    }
}

impl<T> VisitPortableRepr for &mut T
where
    T: VisitPortableRepr + ?Sized,
{
    type Repr = T::Repr;

    fn visit_portable_repr<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Self::Repr) -> R,
    {
        T::visit_portable_repr(self, f)
    }
}

macro_rules! simple_repr {
    ($([$($a:tt)*])* |$arg:ident: &$t:ty| -> &$r:ty { $($body:tt)* }) => {
        impl<$($($a)*),*> VisitPortableRepr for $t {
            type Repr = $r;

            fn visit_portable_repr<F, R>(&self, f: F) -> R
            where
                F: FnOnce(&Self::Repr) -> R,
            {
                let $arg = self;
                let __f = f;
                __f({ $($body)* })
            }
        }
    };
    ($([$($a:tt)*])* |$arg:ident: &$t:ty| -> $r:ty { $($body:tt)* }) => {
        impl<$($($a)*),*> VisitPortableRepr for $t {
            type Repr = $r;

            fn visit_portable_repr<F, R>(&self, f: F) -> R
            where
                F: FnOnce(&Self::Repr) -> R,
            {
                #[allow(unused_variables)]
                let $arg = self;
                let __f = f;
                __f(&{ $($body)* })
            }
        }
    };
}

macro_rules! map_repr {
    ($([$($a:tt)*])* |$arg:ident: &$t:ty| -> &$r:ty { $($body:tt)* }) => {
        impl<$($($a)*),*> VisitPortableRepr for $t {
            type Repr = <$r as VisitPortableRepr>::Repr;

            fn visit_portable_repr<F, R>(&self, f: F) -> R
            where
                F: FnOnce(&Self::Repr) -> R,
            {
                let $arg = self;
                let __f = f;
                <$r as VisitPortableRepr>::visit_portable_repr({ $($body)* }, __f)
            }
        }
    };
    ($([$($a:tt)*])* |$arg:ident: &$t:ty| -> $r:ty { $($body:tt)* }) => {
        impl<$($($a)*),*> VisitPortableRepr for $t {
            type Repr = <$r as VisitPortableRepr>::Repr;

            fn visit_portable_repr<F, R>(&self, f: F) -> R
            where
                F: FnOnce(&Self::Repr) -> R,
            {
                #[allow(unused_variables)]
                let $arg = self;
                let __f = f;
                <$r as VisitPortableRepr>::visit_portable_repr(&{ $($body)* }, __f)
            }
        }
    };
}

macro_rules! self_repr {
    ($($p:ident)::* $(<$($t:ident $(: ?$s:ident)?),* $(,)?>)?) => {
        impl $(<$($t $(: ?$s)?),*>)? VisitPortableRepr for $($p)::* $(<$($t),*>)? {
            type Repr = Self;

            fn visit_portable_repr<F, R>(&self, f: F) -> R
            where
                F: FnOnce(&Self::Repr) -> R,
            {
                f(self)
            }
        }
    }
}

macro_rules! ints {
    ($(($i:ident, $le:ident, $be:ident, $nzl:ident, $nzb:ident)),* $(,)?) => {
        $(
            self_repr!($i);

            simple_repr!(|val: &core::num::NonZero<$i>| -> $i { (*val).get() });

            cfg_select!(feature = "rend-0_5" => {
                simple_repr!(|val: &rend_0_5::$le| -> $i { val.to_native() });
                simple_repr!(|val: &rend_0_5::$be| -> $i { val.to_native() });
                simple_repr!(|val: &rend_0_5::$nzl| -> $i { val.to_native().get() });
                simple_repr!(|val: &rend_0_5::$nzb| -> $i { val.to_native().get() });
            } _ => {});
        )*
    }
}

ints! {
    (u16, u16_le, u16_be, NonZeroU16_le, NonZeroU16_be),
    (u32, u32_le, u32_be, NonZeroU32_le, NonZeroU32_be),
    (u64, u64_le, u64_be, NonZeroU64_le, NonZeroU64_be),
    (u128, u128_le, u128_be, NonZeroU128_le, NonZeroU128_be),
    (i16, i16_le, i16_be, NonZeroI16_le, NonZeroI16_be),
    (i32, i32_le, i32_be, NonZeroI32_le, NonZeroI32_be),
    (i64, i64_le, i64_be, NonZeroI64_le, NonZeroI64_be),
    (i128, i128_le, i128_be, NonZeroI128_le, NonZeroI128_be),
}

self_repr!(bool);
self_repr!(u8);
self_repr!(i8);
self_repr!(usize);
self_repr!(isize);
self_repr!(char);

simple_repr!(|val: &core::num::NonZeroU8| -> u8 { (*val).get() });
simple_repr!(|val: &core::num::NonZeroI8| -> i8 { (*val).get() });
simple_repr!(|val: &core::num::NonZeroUsize| -> usize { (*val).get() });
simple_repr!(|val: &core::num::NonZeroIsize| -> isize { (*val).get() });

cfg_select!(feature = "rend-0_5" => {
    simple_repr!(|val: &rend_0_5::char_le| -> char { val.to_native() });
    simple_repr!(|val: &rend_0_5::char_be| -> char { val.to_native() });
} _ => {});

impl VisitPortableRepr for () {
    type Repr = Self;

    fn visit_portable_repr<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Self::Repr) -> R,
    {
        f(&())
    }
}

macro_rules! tuple {
    ($n0:ident $([$r0:ident])? $t0:ident $(, $nn:ident $([$rn:ident])? $tn:ident)* $(,)?) => {
        tuple! { { $n0 [$($r0)?] $t0 } { $(($nn $tn $($rn)?))* } }
    };
    ({ $n:ident [$($r:ident)?] $($t:ident)* } { ($n0:ident $t0:ident $($r0:ident)?) $(($nn:ident $tn:ident $($rn:ident)?))* }) => {
        tuple! { { $n [$($r)?] $($t)* } { } }
        tuple! { { $n0 [$($r0)?] $($t)* $t0 } { $(($nn $tn $($rn)?))* } }
    };
    ({ $n:ident [$($r:ident)?] $($t:ident)* } { }) => {
        /// Portable representation of a tuple, holding a borrowed view of each element.
        ///
        /// Tuples are represented by the type of matching arity, `Tuple1Repr` through
        /// `Tuple16Repr`, and are compared element by element.
        pub struct $n<$($t: ?Sized),*>($(NonNull<$t>),*);

        impl<$($t: ?Sized),*> $n<$($t),*> {
            /// Calls `f` with the representation of a tuple of references.
            pub fn visit<F, R>(tuple: ($(&$t,)*), f: F) -> R
            where
                F: FnOnce(&Self) -> R,
            {
                #[allow(non_snake_case)]
                fn visit<$($t: ?Sized,)* F, R>(
                    ($($t,)*): ($(&$t,)*),
                    f: F,
                ) -> R
                where
                    F: FnOnce(&$n<$($t),*>) -> R,
                {
                    f(&$n($(NonNull::from_ref($t)),*))
                }

                visit(tuple, f)
            }

            /// Returns the represented elements as a tuple of references.
            pub fn as_ref(&self) -> ($(&$t,)*) {
                #[allow(non_snake_case)]
                fn make_ref<$($t: ?Sized),*>(
                    $n($($t),*): &$n<$($t),*>,
                ) -> ($(&$t,)*) {
                    // SAFETY: the pointers are only ever set by `visit`, from references that
                    // outlive the call it lends `Self` out for, and the returned borrows are
                    // tied to `&self`, so they cannot outlive those references.
                    unsafe {
                        ($($t.as_ref(),)*)
                    }
                }

                make_ref(self)
            }
        }

        self_repr!($n<$($t: ?Sized),*>);

        impl<$($t),*> VisitPortableRepr for ($($t,)*) {
            type Repr = $n<$($t),*>;

            fn visit_portable_repr<F, R>(&self, f: F) -> R
            where
                F: FnOnce(&Self::Repr) -> R,
            {
                #[allow(non_snake_case)]
                fn visit<$($t,)* F, R>(
                    ($($t,)*): &($($t,)*),
                    f: F,
                ) -> R
                where
                    F: FnOnce(&$n<$($t),*>) -> R,
                {
                    $n::<$($t),*>::visit(($($t,)*), f)
                }

                visit(self, f)
            }
        }

        tuple! { @rkyv $n [$($r)?] $($t)* }
    };
    (@rkyv $n:ident [$r:ident] $($t:ident)*) => {
        #[cfg(feature = "rkyv-0_8")]
        impl<$($t),*> VisitPortableRepr for rkyv_0_8::tuple::$r<$($t),*> {
            type Repr = $n<$($t),*>;

            fn visit_portable_repr<F, R>(&self, f: F) -> R
            where
                F: FnOnce(&Self::Repr) -> R,
            {
                #[allow(non_snake_case)]
                fn visit<$($t,)* F, R>(
                    rkyv_0_8::tuple::$r($($t,)*): &rkyv_0_8::tuple::$r<$($t),*>,
                    f: F,
                ) -> R
                where
                    F: FnOnce(&$n<$($t),*>) -> R,
                {
                    $n::<$($t),*>::visit(($($t,)*), f)
                }

                visit(self, f)
            }
        }
    };
    (@rkyv $n:ident [] $($t:ident)*) => {};
}

tuple! {
    Tuple1Repr [ArchivedTuple1] T0,
    Tuple2Repr [ArchivedTuple2] T1,
    Tuple3Repr [ArchivedTuple3] T2,
    Tuple4Repr [ArchivedTuple4] T3,
    Tuple5Repr [ArchivedTuple5] T4,
    Tuple6Repr [ArchivedTuple6] T5,
    Tuple7Repr [ArchivedTuple7] T6,
    Tuple8Repr [ArchivedTuple8] T7,
    Tuple9Repr [ArchivedTuple9] T8,
    Tuple10Repr [ArchivedTuple10] T9,
    Tuple11Repr [ArchivedTuple11] T10,
    Tuple12Repr [ArchivedTuple12] T11,
    Tuple13Repr [ArchivedTuple13] T12,
    Tuple14Repr T13,
    Tuple15Repr T14,
    Tuple16Repr T16,
}

impl<T> VisitPortableRepr for [T] {
    type Repr = Self;

    fn visit_portable_repr<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Self::Repr) -> R,
    {
        f(self)
    }
}

impl<T, const N: usize> VisitPortableRepr for [T; N] {
    type Repr = [T];

    fn visit_portable_repr<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Self::Repr) -> R,
    {
        f(self)
    }
}

self_repr!(str);
self_repr!(core::ffi::CStr);
self_repr!(core::convert::Infallible);
self_repr!(core::marker::PhantomData<T: ?Sized>);
self_repr!(core::marker::PhantomPinned);
map_repr!([T: VisitPortableRepr] |val: &core::panic::AssertUnwindSafe<T>| -> &T { &val.0 });
simple_repr!(|ip: &core::net::Ipv4Addr| -> core::net::IpAddr { (*ip).into() });
simple_repr!(|ip: &core::net::Ipv6Addr| -> core::net::IpAddr { (*ip).into() });
self_repr!(core::net::IpAddr);
simple_repr!(|addr: &core::net::SocketAddrV4| -> core::net::SocketAddr { (*addr).into() });
simple_repr!(|addr: &core::net::SocketAddrV6| -> core::net::SocketAddr { (*addr).into() });
self_repr!(core::net::SocketAddr);
map_repr!([T: VisitPortableRepr] |val: &core::num::Saturating<T>| -> &T { &val.0 });
map_repr!([T: VisitPortableRepr] |val: &core::num::Wrapping<T>| -> &T { &val.0 });
self_repr!(core::cmp::Reverse<T>);
self_repr!(core::cmp::Ordering);
map_repr!([P: core::ops::Deref<Target: VisitPortableRepr>] |val: &core::pin::Pin<P>| -> &P::Target { val });
self_repr!(core::task::Poll<T>);
self_repr!(core::time::Duration);

/// Portable representation of optional values, holding a borrowed view of the contained value.
///
/// Shared by every optional type — `Option<T>` and its niched and archived counterparts — so
/// they compare with each other. A `None` sorts before any `Some`.
///
/// # Example
///
/// ```
/// use portable::repr::{OptionRepr, VisitPortableRepr};
///
/// let doubled = Some(21u32).visit_portable_repr(|repr| repr.as_ref().map(|v| v * 2));
///
/// assert_eq!(doubled, Some(42));
/// ```
pub struct OptionRepr<T: ?Sized>(Option<NonNull<T>>);

impl<T: ?Sized> OptionRepr<T> {
    /// Calls `f` with the representation of an optional reference.
    ///
    /// # Example
    ///
    /// ```
    /// use portable::repr::OptionRepr;
    ///
    /// let is_some = OptionRepr::visit(Some(&1u32), |repr| repr.as_ref().is_some());
    ///
    /// assert!(is_some);
    /// ```
    pub fn visit<F, R>(opt: Option<&T>, f: F) -> R
    where
        F: FnOnce(&Self) -> R,
    {
        f(&Self(opt.map(NonNull::from_ref)))
    }

    /// Returns the represented value as an optional reference.
    ///
    /// # Example
    ///
    /// ```
    /// use portable::repr::OptionRepr;
    ///
    /// let value = OptionRepr::visit(Some(&1u32), |repr| repr.as_ref().copied());
    ///
    /// assert_eq!(value, Some(1));
    /// ```
    pub fn as_ref(&self) -> Option<&T> {
        // SAFETY: the pointer is only ever set by `visit`, from a reference that outlives the
        // call it lends `Self` out for, and the returned borrow is tied to `&self`, so it
        // cannot outlive that reference.
        unsafe { self.0.map(|ptr| ptr.as_ref()) }
    }
}

self_repr!(OptionRepr<T: ?Sized>);

impl<T> VisitPortableRepr for Option<T> {
    type Repr = OptionRepr<T>;

    fn visit_portable_repr<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Self::Repr) -> R,
    {
        OptionRepr::<T>::visit(self.as_ref(), f)
    }
}

/// Portable representation of fallible values, holding a borrowed view of the contained value.
///
/// Shared by `Result<T, E>` and its archived counterparts, so they compare with each other. An
/// `Ok` sorts before any `Err`.
///
/// # Example
///
/// ```
/// use portable::repr::{ResultRepr, VisitPortableRepr};
///
/// let value: Result<u32, ()> = Ok(21);
/// let doubled = value.visit_portable_repr(|repr| repr.as_ref().map(|v| v * 2).map_err(|_| ()));
///
/// assert_eq!(doubled, Ok(42));
/// ```
pub struct ResultRepr<T: ?Sized, E: ?Sized>(Result<NonNull<T>, NonNull<E>>);

impl<T: ?Sized, E: ?Sized> ResultRepr<T, E> {
    /// Calls `f` with the representation of a result of references.
    ///
    /// # Example
    ///
    /// ```
    /// use portable::repr::ResultRepr;
    ///
    /// let result: Result<&u32, &()> = Ok(&1);
    /// let is_ok = ResultRepr::visit(result, |repr| repr.as_ref().is_ok());
    ///
    /// assert!(is_ok);
    /// ```
    pub fn visit<F, R>(res: Result<&T, &E>, f: F) -> R
    where
        F: FnOnce(&Self) -> R,
    {
        f(&Self(res.map(NonNull::from_ref).map_err(NonNull::from_ref)))
    }

    /// Returns the represented value as a result of references.
    ///
    /// # Example
    ///
    /// ```
    /// use portable::repr::ResultRepr;
    ///
    /// let result: Result<&u32, &()> = Ok(&1);
    /// let value = ResultRepr::visit(result, |repr| repr.as_ref().ok().copied());
    ///
    /// assert_eq!(value, Some(1));
    /// ```
    pub fn as_ref(&self) -> Result<&T, &E> {
        // SAFETY: the pointer is only ever set by `visit`, from a reference that outlives the
        // call it lends `Self` out for, and the returned borrow is tied to `&self`, so it
        // cannot outlive that reference.
        unsafe { self.0.map(|ptr| ptr.as_ref()).map_err(|ptr| ptr.as_ref()) }
    }
}

self_repr!(ResultRepr<T: ?Sized, E: ?Sized>);

impl<T, E> VisitPortableRepr for Result<T, E> {
    type Repr = ResultRepr<T, E>;

    fn visit_portable_repr<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Self::Repr) -> R,
    {
        ResultRepr::<T, E>::visit(self.as_ref(), f)
    }
}

/// Portable representation of a range bound, holding a borrowed view of the bound value.
///
/// # Example
///
/// ```
/// use core::ops::Bound;
/// use portable::repr::{BoundRepr, VisitPortableRepr};
///
/// let bound = Bound::Included(1u32);
/// let value = bound.visit_portable_repr(|repr| repr.as_ref().cloned());
///
/// assert_eq!(value, Bound::Included(1));
/// ```
pub struct BoundRepr<T: ?Sized>(core::ops::Bound<NonNull<T>>);

impl<T: ?Sized> BoundRepr<T> {
    /// Calls `f` with the representation of a bound holding a reference.
    ///
    /// # Example
    ///
    /// ```
    /// use core::ops::Bound;
    /// use portable::repr::BoundRepr;
    ///
    /// let is_unbounded = BoundRepr::visit(Bound::Included(&1u32), |repr| {
    ///     matches!(repr.as_ref(), Bound::Unbounded)
    /// });
    ///
    /// assert!(!is_unbounded);
    /// ```
    pub fn visit<F, R>(bound: core::ops::Bound<&T>, f: F) -> R
    where
        F: FnOnce(&Self) -> R,
    {
        f(&Self(bound.map(NonNull::from_ref)))
    }

    /// Returns the represented bound with a reference to its value.
    ///
    /// # Example
    ///
    /// ```
    /// use core::ops::Bound;
    /// use portable::repr::BoundRepr;
    ///
    /// let bound = BoundRepr::visit(Bound::Included(&1u32), |repr| repr.as_ref().cloned());
    ///
    /// assert_eq!(bound, Bound::Included(1));
    /// ```
    pub fn as_ref(&self) -> core::ops::Bound<&T> {
        // SAFETY: the pointer is only ever set by `visit`, from a reference that outlives the
        // call it lends `Self` out for, and the returned borrow is tied to `&self`, so it
        // cannot outlive that reference.
        unsafe { self.0.map(|ptr| ptr.as_ref()) }
    }
}

self_repr!(BoundRepr<T: ?Sized>);

impl<T> VisitPortableRepr for core::ops::Bound<T> {
    type Repr = BoundRepr<T>;

    fn visit_portable_repr<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Self::Repr) -> R,
    {
        BoundRepr::<T>::visit(self.as_ref(), f)
    }
}

/// Portable representation of ranges: their start and end [`BoundRepr`]s as a pair.
///
/// Every range type shares this representation and so compares against the others;
/// `RangeFull`, which has no bound values, is represented as
/// `RangeRepr<Infallible>`.
///
/// # Example
///
/// ```
/// use core::ops::Bound;
/// use portable::repr::{RangeRepr, VisitPortableRepr};
///
/// (1u32..3).visit_portable_repr(|repr: &RangeRepr<u32>| {
///     let (start, end) = repr.as_ref();
///
///     assert_eq!(start.as_ref(), Bound::Included(&1));
///     assert_eq!(end.as_ref(), Bound::Excluded(&3));
/// });
/// ```
pub type RangeRepr<T> = Tuple2Repr<BoundRepr<T>, BoundRepr<T>>;

fn visit_range_repr<T, R, F, O>(range: &R, f: F) -> O
where
    R: core::ops::RangeBounds<T>,
    F: FnOnce(&RangeRepr<T>) -> O,
{
    let start = range.start_bound();
    let end = range.end_bound();
    BoundRepr::<T>::visit(start, move |start| {
        BoundRepr::<T>::visit(end, move |end| {
            Tuple2Repr::<BoundRepr<T>, BoundRepr<T>>::visit((start, end), f)
        })
    })
}

macro_rules! range {
    ($($($p:ident)::+),* $(,)?) => {
        $(
            impl<T> VisitPortableRepr for $($p)::+ <T> {
                type Repr = RangeRepr<T>;

                fn visit_portable_repr<F, R>(&self, f: F) -> R
                where
                    F: FnOnce(&Self::Repr) -> R,
                {
                    visit_range_repr(self, f)
                }
            }
        )*
    }
}

range! {
    core::ops::Range,
    core::ops::RangeInclusive,
    core::ops::RangeFrom,
    core::ops::RangeTo,
    core::ops::RangeToInclusive,
    core::range::Range,
    core::range::RangeInclusive,
    core::range::RangeFrom,
    core::range::RangeToInclusive,
}

impl VisitPortableRepr for core::ops::RangeFull {
    type Repr = RangeRepr<core::convert::Infallible>;

    fn visit_portable_repr<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Self::Repr) -> R,
    {
        visit_range_repr(self, f)
    }
}

cfg_select!(feature = "alloc" => {
    map_repr!([T: VisitPortableRepr + ?Sized] |val: &alloc::boxed::Box<T>| -> &T { val });
    map_repr!([T: VisitPortableRepr + ?Sized] |val: &alloc::rc::Rc<T>| -> &T { val });
    map_repr!([T: VisitPortableRepr + ?Sized] |val: &alloc::sync::Arc<T>| -> &T { val });
    map_repr!([T] |val: &alloc::vec::Vec<T>| -> &[T] { val });
    map_repr!(|val: &alloc::string::String| -> &str { val });
    map_repr!(|val: &alloc::ffi::CString| -> &core::ffi::CStr { val });
    map_repr!([T: VisitPortableRepr + alloc::borrow::ToOwned + ?Sized] |val: &alloc::borrow::Cow<'_, T>| -> &T { val });

    self_repr!(alloc::collections::BTreeSet<T>);
    self_repr!(alloc::collections::BTreeMap<K, V>);
    self_repr!(alloc::collections::LinkedList<T>);
    self_repr!(alloc::collections::VecDeque<T>);
} _ => {});

cfg_select!(feature = "rkyv-0_8" => {
    map_repr!([T: VisitPortableRepr + ?Sized] |val: &rkyv_0_8::seal::Seal<'_, T>| -> &T { val });

    map_repr! {
        [T: VisitPortableRepr + rkyv_0_8::traits::ArchivePointee + ?Sized]
        |val: &rkyv_0_8::boxed::ArchivedBox<T>| -> &T { val }
    }

    impl<T, const E: usize> VisitPortableRepr
        for rkyv_0_8::collections::btree_set::ArchivedBTreeSet<T, E>
    {
        type Repr = Self;

        fn visit_portable_repr<F, R>(&self, f: F) -> R
        where
            F: FnOnce(&Self::Repr) -> R,
        {
            f(self)
        }
    }

    impl<K, V, const E: usize> VisitPortableRepr
        for rkyv_0_8::collections::btree_map::ArchivedBTreeMap<K, V, E>
    {
        type Repr = Self;

        fn visit_portable_repr<F, R>(&self, f: F) -> R
        where
            F: FnOnce(&Self::Repr) -> R,
        {
            f(self)
        }
    }

    simple_repr!(|val: &rkyv_0_8::ffi::ArchivedCString| -> &core::ffi::CStr { val.as_c_str() });
    map_repr!(|val: &rkyv_0_8::net::ArchivedIpv4Addr| -> core::net::Ipv4Addr { val.as_ipv4() });
    map_repr!(|val: &rkyv_0_8::net::ArchivedIpv6Addr| -> core::net::Ipv6Addr { val.as_ipv6() });
    simple_repr!(|val: &rkyv_0_8::net::ArchivedIpAddr| -> core::net::IpAddr { val.as_ipaddr() });
    map_repr!(|val: &rkyv_0_8::net::ArchivedSocketAddrV4| -> core::net::SocketAddrV4 { val.as_socket_addr_v4() });
    map_repr!(|val: &rkyv_0_8::net::ArchivedSocketAddrV6| -> core::net::SocketAddrV6 { val.as_socket_addr_v6() });
    map_repr!(|val: &rkyv_0_8::net::ArchivedSocketAddr| -> core::net::SocketAddr { val.as_socket_addr() });

    impl<T, N> VisitPortableRepr for rkyv_0_8::niche::niched_option::NichedOption<T, N>
    where
        N: rkyv_0_8::niche::niching::Niching<T> + ?Sized,
    {
        type Repr = OptionRepr<T>;

        fn visit_portable_repr<F, R>(&self, f: F) -> R
        where
            F: FnOnce(&Self::Repr) -> R,
        {
            OptionRepr::<T>::visit(self.as_ref(), f)
        }
    }

    impl<T> VisitPortableRepr for rkyv_0_8::niche::option_box::ArchivedOptionBox<T>
    where
        T: rkyv_0_8::traits::ArchivePointee + ?Sized,
    {
        type Repr = OptionRepr<T>;

        fn visit_portable_repr<F, R>(&self, f: F) -> R
        where
            F: FnOnce(&Self::Repr) -> R,
        {
            OptionRepr::<T>::visit(self.as_deref(), f)
        }
    }

    macro_rules! option_nonzero {
        ($($t:ident => $r:ident),* $(,)?) => {
            $(
                impl VisitPortableRepr for rkyv_0_8::niche::option_nonzero::$t {
                    type Repr = OptionRepr<rkyv_0_8::primitive::$r>;

                    fn visit_portable_repr<F, R>(&self, f: F) -> R
                    where
                        F: FnOnce(&Self::Repr) -> R,
                    {
                        OptionRepr::<rkyv_0_8::primitive::$r>::visit(self.as_ref(), f)
                    }
                }
            )*
        }
    }

    option_nonzero! {
        ArchivedOptionNonZeroU16 => ArchivedNonZeroU16,
        ArchivedOptionNonZeroU32 => ArchivedNonZeroU32,
        ArchivedOptionNonZeroU64 => ArchivedNonZeroU64,
        ArchivedOptionNonZeroU128 => ArchivedNonZeroU128,
        ArchivedOptionNonZeroI16 => ArchivedNonZeroI16,
        ArchivedOptionNonZeroI32 => ArchivedNonZeroI32,
        ArchivedOptionNonZeroI64 => ArchivedNonZeroI64,
        ArchivedOptionNonZeroI128 => ArchivedNonZeroI128,
    }

    impl VisitPortableRepr for rkyv_0_8::niche::option_nonzero::ArchivedOptionNonZeroU8 {
        type Repr = OptionRepr<core::num::NonZeroU8>;

        fn visit_portable_repr<F, R>(&self, f: F) -> R
        where
            F: FnOnce(&Self::Repr) -> R,
        {
            OptionRepr::<core::num::NonZeroU8>::visit(self.as_ref(), f)
        }
    }

    impl VisitPortableRepr for rkyv_0_8::niche::option_nonzero::ArchivedOptionNonZeroI8 {
        type Repr = OptionRepr<core::num::NonZeroI8>;

        fn visit_portable_repr<F, R>(&self, f: F) -> R
        where
            F: FnOnce(&Self::Repr) -> R,
        {
            OptionRepr::<core::num::NonZeroI8>::visit(self.as_ref(), f)
        }
    }

    impl<T> VisitPortableRepr for rkyv_0_8::ops::ArchivedBound<T> {
        type Repr = BoundRepr<T>;

        fn visit_portable_repr<F, R>(&self, f: F) -> R
        where
            F: FnOnce(&Self::Repr) -> R,
        {
            BoundRepr::<T>::visit(self.as_ref(), f)
        }
    }

    range! {
        rkyv_0_8::ops::ArchivedRange,
        rkyv_0_8::ops::ArchivedRangeInclusive,
        rkyv_0_8::ops::ArchivedRangeFrom,
        rkyv_0_8::ops::ArchivedRangeTo,
        rkyv_0_8::ops::ArchivedRangeToInclusive,
    }

    map_repr!(|_val: &rkyv_0_8::ops::ArchivedRangeFull| -> core::ops::RangeFull { .. });

    impl<T> VisitPortableRepr for rkyv_0_8::option::ArchivedOption<T> {
        type Repr = OptionRepr<T>;

        fn visit_portable_repr<F, R>(&self, f: F) -> R
        where
            F: FnOnce(&Self::Repr) -> R,
        {
            OptionRepr::<T>::visit(self.as_ref(), f)
        }
    }

    map_repr! {
        [T: VisitPortableRepr + rkyv_0_8::traits::ArchivePointee + ?Sized]
        [L]
        |val: &rkyv_0_8::rc::ArchivedRc<T, L>| -> &T { val }
    }

    impl<T, E> VisitPortableRepr for rkyv_0_8::result::ArchivedResult<T, E> {
        type Repr = ResultRepr<T, E>;

        fn visit_portable_repr<F, R>(&self, f: F) -> R
        where
            F: FnOnce(&Self::Repr) -> R,
        {
            ResultRepr::<T, E>::visit(self.as_ref(), f)
        }
    }

    map_repr!(|val: &rkyv_0_8::string::ArchivedString| -> &str { val.as_str() });

    map_repr!(|val: &rkyv_0_8::time::ArchivedDuration| -> core::time::Duration {
        core::time::Duration::new(val.as_secs(), val.subsec_nanos())
    });

    map_repr!([T: VisitPortableRepr] |val: &rkyv_0_8::util::Align<T>| -> &T { &val.0 });

    #[cfg(feature = "alloc")]
    map_repr!([const ALIGNMENT: usize] |val: &rkyv_0_8::util::AlignedVec<ALIGNMENT>| -> &[u8] { val });

    impl<T, const N: usize> VisitPortableRepr for rkyv_0_8::util::InlineVec<T, N> {
        type Repr = [T];

        fn visit_portable_repr<F, R>(&self, f: F) -> R
        where
            F: FnOnce(&Self::Repr) -> R,
        {
            f(self)
        }
    }

    map_repr!([T] |val: &rkyv_0_8::util::SerVec<T>| -> &[T] { val });
    map_repr!([T] |val: &rkyv_0_8::vec::ArchivedVec<T>| -> &[T] { val });
} _ => {});

cfg_select!(feature = "allocator-api2-0_4" => {
    map_repr! {
        [T: VisitPortableRepr + ?Sized]
        [A: allocator_api2_0_4::alloc::Allocator]
        |val: &allocator_api2_0_4::boxed::Box<T, A>| -> &T { val }
    }

    map_repr! {
        [T: VisitPortableRepr]
        [A: allocator_api2_0_4::alloc::Allocator]
        |val: &allocator_api2_0_4::vec::Vec<T, A>| -> &[T] { val }
    }
} _ => {});

cfg_select!(feature = "allocator-api2-0_3" => {
    map_repr! {
        [T: VisitPortableRepr + ?Sized]
        [A: allocator_api2_0_3::alloc::Allocator]
        |val: &allocator_api2_0_3::boxed::Box<T, A>| -> &T { val }
    }

    map_repr! {
        [T: VisitPortableRepr]
        [A: allocator_api2_0_3::alloc::Allocator]
        |val: &allocator_api2_0_3::vec::Vec<T, A>| -> &[T] { val }
    }
} _ => {});

cfg_select!(feature = "allocator-api2-0_2" => {
    map_repr! {
        [T: VisitPortableRepr + ?Sized]
        [A: allocator_api2_0_2::alloc::Allocator]
        |val: &allocator_api2_0_2::boxed::Box<T, A>| -> &T { val }
    }

    map_repr! {
        [T: VisitPortableRepr]
        [A: allocator_api2_0_2::alloc::Allocator]
        |val: &allocator_api2_0_2::vec::Vec<T, A>| -> &[T] { val }
    }
} _ => {});

cfg_select!(feature = "arrayvec-0_7" => {
    map_repr! {
        [T]
        [const CAP: usize]
        |val: &arrayvec_0_7::ArrayVec<T, CAP>| -> &[T] { val }
    }

    map_repr! {
        [const CAP: usize]
        |val: &arrayvec_0_7::ArrayString<CAP>| -> &str { val }
    }
} _ => {});

cfg_select!(feature = "ascii-1" => {
    map_repr!(|val: &ascii_1::AsciiChar| -> char { (*val).as_char() });
    map_repr!(|val: &ascii_1::AsciiStr| -> &str { val.as_str() });

    #[cfg(feature = "alloc")]
    map_repr!(|val: &ascii_1::AsciiString| -> &str { val.as_str() });
} _ => {});

cfg_select!(feature = "bstr-1" => {
    map_repr!(|val: &bstr_1::BStr| -> &[u8] { val });

    #[cfg(feature = "alloc")]
    map_repr!(|val: &bstr_1::BString| -> &[u8] { val });
} _ => {});

cfg_select!(feature = "bumpalo-3" => {
    map_repr! {
        [T: VisitPortableRepr + ?Sized]
        |val: &bumpalo_3::boxed::Box<'_, T>| -> &T { val }
    }

    map_repr!([T] |val: &bumpalo_3::collections::Vec<'_, T>| -> &[T] { val });
    map_repr!(|val: &bumpalo_3::collections::String<'_>| -> &str { val });
} _ => {});

cfg_select!(feature = "bytes-1" => {
    map_repr!(|val: &bytes_1::Bytes| -> &[u8] { val });
    map_repr!(|val: &bytes_1::BytesMut| -> &[u8] { val });
} _ => {});

cfg_select!(feature = "smallvec-1" => {
    map_repr! {
        [A: smallvec_1::Array]
        |val: &smallvec_1::SmallVec<A>| -> &[A::Item] { val }
    }
} _ => {});

cfg_select!(feature = "smol_str-0_2" => {
    map_repr!(|val: &smol_str_0_2::SmolStr| -> &str { val });
} _ => {});

cfg_select!(feature = "smol_str-0_3" => {
    map_repr!(|val: &smol_str_0_3::SmolStr| -> &str { val });
} _ => {});

cfg_select!(feature = "thin-vec-0_2" => {
    map_repr!([T] |val: &thin_vec_0_2::ThinVec<T>| -> &[T] { val });
} _ => {});

cfg_select!(feature = "tinyvec-1" => {
    map_repr! {
        [A: tinyvec_1::Array]
        |val: &tinyvec_1::ArrayVec<A>| -> &[A::Item] { val }
    }

    map_repr!([T] |val: &tinyvec_1::SliceVec<'_, T>| -> &[T] { val });

    #[cfg(feature = "alloc")]
    map_repr! {
        [A: tinyvec_1::Array]
        |val: &tinyvec_1::TinyVec<A>| -> &[A::Item] { val }
    }
} _ => {});

cfg_select!(feature = "triomphe-0_1" => {
    map_repr! {
        [T: VisitPortableRepr + ?Sized]
        |val: &triomphe_0_1::Arc<T>| -> &T { val }
    }

    map_repr! {
        [T: VisitPortableRepr]
        |val: &triomphe_0_1::ArcBorrow<'_, T>| -> &T { val.get() }
    }

    /// Portable representation of a `triomphe::ArcUnion`, holding a borrowed view of the
    /// contained value.
    ///
    /// A `First` sorts before any `Second`.
    ///
    /// # Example
    ///
    /// ```
    /// use portable::PortableEq;
    /// use triomphe_0_1::{Arc, ArcUnion};
    ///
    /// let a = ArcUnion::<u32, u64>::from_first(Arc::new(1));
    /// let b = ArcUnion::<u32, u64>::from_first(Arc::new(1));
    ///
    /// assert!(a.portable_eq(&b));
    /// ```
    pub struct ArcUnionRepr<A, B>(ArcUnionReprInner<A, B>);

    enum ArcUnionReprInner<A, B> {
        First(NonNull<A>),
        Second(NonNull<B>),
    }

    impl<A, B> ArcUnionRepr<A, B> {
        /// Calls `f` with the representation of a borrowed union.
        ///
        /// # Example
        ///
        /// ```
        /// use portable::repr::ArcUnionRepr;
        /// use triomphe_0_1::{Arc, ArcUnion, ArcUnionBorrow};
        ///
        /// let union = ArcUnion::<u32, u64>::from_first(Arc::new(1));
        /// let is_first = ArcUnionRepr::visit(union.borrow(), |repr| {
        ///     matches!(repr.borrow(), ArcUnionBorrow::First(_))
        /// });
        ///
        /// assert!(is_first);
        /// ```
        pub fn visit<F, R>(arc: triomphe_0_1::ArcUnionBorrow<'_, A, B>, f: F) -> R
        where
            F: FnOnce(&Self) -> R,
        {
            use triomphe_0_1::ArcUnionBorrow::{First, Second};

            f(&Self(match arc {
                First(a) => ArcUnionReprInner::First(NonNull::from_ref(a.get())),
                Second(b) => ArcUnionReprInner::Second(NonNull::from_ref(b.get())),
            }))
        }

        /// Returns the represented union as a borrow of its contained value.
        ///
        /// # Example
        ///
        /// ```
        /// use portable::repr::ArcUnionRepr;
        /// use triomphe_0_1::{Arc, ArcUnion, ArcUnionBorrow};
        ///
        /// let union = ArcUnion::<u32, u64>::from_first(Arc::new(1));
        /// let value = ArcUnionRepr::visit(union.borrow(), |repr| match repr.borrow() {
        ///     ArcUnionBorrow::First(first) => u64::from(*first.get()),
        ///     ArcUnionBorrow::Second(second) => *second.get(),
        /// });
        ///
        /// assert_eq!(value, 1);
        /// ```
        pub fn borrow(&self) -> triomphe_0_1::ArcUnionBorrow<'_, A, B> {
            use triomphe_0_1::ArcUnionBorrow::{First, Second};

            // SAFETY: `ArcBorrow` is `repr(transparent)` over `NonNull<T>`, and the pointers
            // are only ever set by `visit` from a live `ArcBorrow`, which keeps the value
            // alive for as long as the `Self` it builds is lent out. The returned borrow is
            // tied to `&self`, so it cannot outlive that.
            match &self.0 {
                ArcUnionReprInner::First(a) => First(unsafe {
                    core::mem::transmute::<NonNull<A>, triomphe_0_1::ArcBorrow<'_, A>>(*a)
                }),
                ArcUnionReprInner::Second(b) => Second(unsafe {
                    core::mem::transmute::<NonNull<B>, triomphe_0_1::ArcBorrow<'_, B>>(*b)
                }),
            }
        }
    }

    self_repr!(ArcUnionRepr<A, B>);

    impl<A, B> VisitPortableRepr for triomphe_0_1::ArcUnionBorrow<'_, A, B> {
        type Repr = ArcUnionRepr<A, B>;

        fn visit_portable_repr<F, R>(&self, f: F) -> R
        where
            F: FnOnce(&Self::Repr) -> R,
        {
            let arc = match self {
                Self::First(a) => Self::First(*a),
                Self::Second(b) => Self::Second(*b),
            };
            ArcUnionRepr::<A, B>::visit(arc, f)
        }
    }

    impl<A, B> VisitPortableRepr for triomphe_0_1::ArcUnion<A, B> {
        type Repr = ArcUnionRepr<A, B>;

        fn visit_portable_repr<F, R>(&self, f: F) -> R
        where
            F: FnOnce(&Self::Repr) -> R,
        {
            self.borrow().visit_portable_repr(f)
        }
    }

    self_repr!(triomphe_0_1::HeaderSlice<H, T: ?Sized>);

    self_repr!(triomphe_0_1::HeaderWithLength<H>);

    map_repr! {
        [H] [T]
        |val: &triomphe_0_1::ThinArc<H, T>|
            -> &triomphe_0_1::HeaderSlice<triomphe_0_1::HeaderWithLength<H>, [T]>
        { val }
    }

    map_repr! {
        [T: VisitPortableRepr + ?Sized]
        |val: &triomphe_0_1::OffsetArc<T>| -> &T { val }
    }

    map_repr! {
        [T: VisitPortableRepr + ?Sized]
        |val: &triomphe_0_1::UniqueArc<T>| -> &T { val }
    }
} _ => {});

cfg_select!(feature = "either-1" => {
    impl<L, R> VisitPortableRepr for either_1::Either<L, R> {
        type Repr = Self;

        fn visit_portable_repr<F, O>(&self, f: F) -> O
        where
            F: FnOnce(&Self::Repr) -> O,
        {
            f(self)
        }
    }
} _ => {});

#[cfg(test)]
mod tests {
    use super::{
        BoundRepr, OptionRepr, RangeRepr, ResultRepr, Tuple1Repr, Tuple2Repr, VisitPortableRepr,
    };

    use core::ops::Bound;

    #[test]
    fn option_repr_round_trips_unsized_values() {
        OptionRepr::visit(Some("hello"), |repr| {
            assert_eq!(repr.as_ref(), Some("hello"));
        });
        OptionRepr::<str>::visit(None, |repr| {
            assert_eq!(repr.as_ref(), None);
        });
    }

    #[test]
    fn result_repr_round_trips_both_variants() {
        let ok: Result<&u32, &str> = Ok(&1);
        ResultRepr::visit(ok, |repr| assert_eq!(repr.as_ref(), Ok(&1)));

        let err: Result<&u32, &str> = Err("bad");
        ResultRepr::visit(err, |repr| assert_eq!(repr.as_ref(), Err("bad")));
    }

    #[test]
    fn bound_repr_round_trips_all_variants() {
        BoundRepr::visit(Bound::Included(&1u32), |repr| {
            assert_eq!(repr.as_ref(), Bound::Included(&1));
        });
        BoundRepr::visit(Bound::Excluded(&1u32), |repr| {
            assert_eq!(repr.as_ref(), Bound::Excluded(&1));
        });
        BoundRepr::<u32>::visit(Bound::Unbounded, |repr| {
            assert_eq!(repr.as_ref(), Bound::Unbounded);
        });
    }

    #[test]
    fn tuple_repr_round_trips_unsized_elements() {
        Tuple2Repr::<str, [u32]>::visit(("hi", &[1u32, 2][..]), |repr| {
            let (text, values) = repr.as_ref();

            assert_eq!(text, "hi");
            assert_eq!(values, &[1, 2]);
        });
    }

    #[test]
    fn tuple_repr_round_trips_at_the_arity_bounds() {
        (1u32,).visit_portable_repr(|repr: &Tuple1Repr<u32>| {
            assert_eq!(*repr.as_ref().0, 1);
        });

        let widest = (
            0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 9u8, 10u8, 11u8, 12u8, 13u8, 14u8, 15u8,
        );

        widest.visit_portable_repr(|repr| {
            let elements = repr.as_ref();

            assert_eq!(*elements.0, 0);
            assert_eq!(*elements.7, 7);
            assert_eq!(*elements.15, 15);
        });
    }

    fn range_bounds<R>(range: R) -> (Bound<u32>, Bound<u32>)
    where
        R: VisitPortableRepr<Repr = RangeRepr<u32>>,
    {
        range.visit_portable_repr(|repr| {
            let (start, end) = repr.as_ref();

            (start.as_ref().cloned(), end.as_ref().cloned())
        })
    }

    #[test]
    fn every_range_kind_reports_its_bounds() {
        assert_eq!(
            range_bounds(1u32..3),
            (Bound::Included(1), Bound::Excluded(3))
        );
        assert_eq!(
            range_bounds(1u32..=3),
            (Bound::Included(1), Bound::Included(3))
        );
        assert_eq!(range_bounds(1u32..), (Bound::Included(1), Bound::Unbounded));
        assert_eq!(range_bounds(..3u32), (Bound::Unbounded, Bound::Excluded(3)));
        assert_eq!(
            range_bounds(..=3u32),
            (Bound::Unbounded, Bound::Included(3))
        );
    }

    #[test]
    fn range_full_is_unbounded_at_both_ends() {
        (..).visit_portable_repr(|repr: &RangeRepr<core::convert::Infallible>| {
            let (start, end) = repr.as_ref();

            assert!(matches!(start.as_ref(), Bound::Unbounded));
            assert!(matches!(end.as_ref(), Bound::Unbounded));
        });
    }

    #[test]
    fn wrappers_forward_to_the_inner_representation() {
        let value = core::num::NonZeroU32::new(7).unwrap();

        assert_eq!(value.visit_portable_repr(|repr| *repr), 7u32);
        assert_eq!(
            crate::Portable(value).visit_portable_repr(|repr| *repr),
            7u32
        );
        assert_eq!(
            core::num::Wrapping(7u32).visit_portable_repr(|repr| *repr),
            7u32
        );
        assert_eq!([1u32, 2, 3].visit_portable_repr(<[u32]>::len), 3);
    }

    #[cfg(feature = "triomphe-0_1")]
    #[test]
    fn arc_union_repr_round_trips_both_variants() {
        use super::ArcUnionRepr;
        use triomphe_0_1::{Arc, ArcUnion, ArcUnionBorrow};

        let first = ArcUnion::<u32, u64>::from_first(Arc::new(1));
        ArcUnionRepr::visit(first.borrow(), |repr| match repr.borrow() {
            ArcUnionBorrow::First(value) => assert_eq!(*value.get(), 1),
            ArcUnionBorrow::Second(_) => panic!("expected first"),
        });

        let second = ArcUnion::<u32, u64>::from_second(Arc::new(2));
        ArcUnionRepr::visit(second.borrow(), |repr| match repr.borrow() {
            ArcUnionBorrow::First(_) => panic!("expected second"),
            ArcUnionBorrow::Second(value) => assert_eq!(*value.get(), 2),
        });
    }
}
