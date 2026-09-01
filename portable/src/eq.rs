//! [`PortableEq`], the platform-independent counterpart of [`Eq`].

use super::repr::{self, PortableRepr, VisitPortableRepr};

/// Equality that gives the same answer on every platform, across types.
///
/// Blanket-implemented for every [`VisitPortableRepr`] type whose representation implements
/// [`PortableReprEq`] for the other type's representation, so any two types sharing a
/// representation compare with each other.
///
/// Implementations must uphold the contract with [`PortableHash`](crate::PortableHash): equal
/// values must hash equally.
///
/// # Example
///
/// ```
/// use portable::PortableEq;
///
/// let slice: &[u32] = &[1, 2, 3];
///
/// assert!(1usize.portable_eq(&1u64));
/// assert!([1u32, 2, 3].portable_eq(slice));
/// assert!(Some(1usize).portable_eq(&Some(1u64)));
/// ```
pub trait PortableEq<K: ?Sized = Self> {
    /// Returns `true` if the two values are equal.
    ///
    /// # Example
    ///
    /// ```
    /// use portable::PortableEq;
    ///
    /// assert!(1usize.portable_eq(&1u64));
    /// assert!(!1usize.portable_eq(&2u64));
    /// ```
    fn portable_eq(&self, other: &K) -> bool;
}

/// Equality between two [portable representations](PortableRepr).
///
/// This is the trait to implement for a representation type; [`PortableEq`] on the types that
/// share those representations follows from it. Implementing it for
/// [`Infallible`](core::convert::Infallible) as well makes a representation comparable with
/// representations that can never hold a value, such as the bounds of a `RangeFull`.
///
/// # Example
///
/// ```
/// use portable::eq::PortableReprEq;
///
/// // `usize` compares against every width it may be stored as.
/// assert!(1usize.portable_repr_eq(&1u64));
/// ```
pub trait PortableReprEq<K: PortableRepr + ?Sized = Self>: PortableRepr {
    /// Returns `true` if the two representations are equal.
    ///
    /// # Example
    ///
    /// ```
    /// use portable::eq::PortableReprEq;
    ///
    /// assert!(1usize.portable_repr_eq(&1u64));
    /// assert!(!1usize.portable_repr_eq(&2u64));
    /// ```
    fn portable_repr_eq(&self, other: &K) -> bool;
}

impl<T, U> PartialEq<super::Portable<U>> for super::Portable<T>
where
    T: PortableEq<U> + ?Sized,
    U: ?Sized,
{
    fn eq(&self, other: &super::Portable<U>) -> bool {
        self.0.portable_eq(&other.0)
    }
}

impl<T> Eq for super::Portable<T> where T: PortableEq + ?Sized {}

impl<T, U> PortableEq<U> for T
where
    T: VisitPortableRepr + ?Sized,
    U: VisitPortableRepr + ?Sized,
    T::Repr: PortableRepr + PortableReprEq<U::Repr>,
    U::Repr: PortableRepr,
{
    fn portable_eq(&self, other: &U) -> bool {
        self.visit_portable_repr(move |l| other.visit_portable_repr(move |r| l.portable_repr_eq(r)))
    }
}

impl<K: PortableEq + PortableRepr + ?Sized> PortableReprEq<K> for core::convert::Infallible {
    fn portable_repr_eq(&self, other: &K) -> bool {
        let _ = other;
        true
    }
}

impl<T> PortableReprEq for crate::AssertPortable<T>
where
    T: Eq + ?Sized,
{
    fn portable_repr_eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> PortableReprEq<core::convert::Infallible> for crate::AssertPortable<T>
where
    T: Eq + ?Sized,
{
    fn portable_repr_eq(&self, other: &core::convert::Infallible) -> bool {
        let _ = other;
        true
    }
}

macro_rules! eq_self {
    ($t:ty) => {
        impl PortableReprEq for $t {
            #[inline]
            fn portable_repr_eq(&self, other: &Self) -> bool {
                self == other
            }
        }

        impl PortableReprEq<core::convert::Infallible> for $t {
            #[inline]
            fn portable_repr_eq(&self, other: &core::convert::Infallible) -> bool {
                let _ = other;
                true
            }
        }
    };
}

eq_self!(());
eq_self!(bool);
eq_self!(char);
eq_self!(u8);
eq_self!(u16);
eq_self!(u32);
eq_self!(u64);
eq_self!(u128);
eq_self!(usize);
eq_self!(i8);
eq_self!(i16);
eq_self!(i32);
eq_self!(i64);
eq_self!(i128);
eq_self!(isize);
eq_self!(str);
eq_self!(core::cmp::Ordering);
eq_self!(core::ffi::CStr);
eq_self!(core::net::IpAddr);
eq_self!(core::net::SocketAddr);
eq_self!(core::time::Duration);

macro_rules! size_types {
    ($($t:ident [$r:ident] { $($i:ident),* $(,)? }),* $(,)?) => {
        $($(
            impl PortableReprEq<$i> for $t {
                #[inline]
                fn portable_repr_eq(&self, other: &$i) -> bool {
                    (*self as $r) == (*other as $r)
                }
            }

            impl PortableReprEq<$t> for $i {
                #[inline]
                fn portable_repr_eq(&self, other: &$t) -> bool {
                    (*self as $r) == (*other as $r)
                }
            }
        )*)*
    }
}

// A `usize` may be serialized as any fixed width, so it must compare against all of
// them for results to be independent of the format that produced the value.
size_types! {
    usize [u64] { u16, u32, u64 },
    isize [i64] { i16, i32, i64 },
}

impl<T, U> PortableReprEq<[U]> for [T]
where
    T: PortableEq<U>,
{
    fn portable_repr_eq(&self, other: &[U]) -> bool {
        self.len() == other.len() && self.iter().zip(other).all(|(l, r)| l.portable_eq(r))
    }
}

impl<T> PortableReprEq<core::convert::Infallible> for [T]
where
    T: PortableEq,
{
    fn portable_repr_eq(&self, other: &core::convert::Infallible) -> bool {
        let _ = other;
        true
    }
}

#[cfg(feature = "alloc")]
macro_rules! seq_eq {
    ([$($lg:tt)*] $l:ty, [$($rg:tt)*] $r:ty) => {
        impl<$($lg)*, $($rg)*> PortableReprEq<$r> for $l
        where
            T: PortableEq<U>,
        {
            fn portable_repr_eq(&self, other: &$r) -> bool {
                self.len() == other.len()
                    && self.iter().zip(other.iter()).all(|(l, r)| l.portable_eq(r))
            }
        }

        impl<$($lg)*, U> PortableReprEq<[U]> for $l
        where
            T: PortableEq<U>,
        {
            fn portable_repr_eq(&self, other: &[U]) -> bool {
                self.len() == other.len()
                    && self.iter().zip(other.iter()).all(|(l, r)| l.portable_eq(r))
            }
        }

        impl<$($rg)*, T> PortableReprEq<$r> for [T]
        where
            T: PortableEq<U>,
        {
            fn portable_repr_eq(&self, other: &$r) -> bool {
                self.len() == other.len()
                    && self.iter().zip(other.iter()).all(|(l, r)| l.portable_eq(r))
            }
        }

        impl<$($lg)*> PortableReprEq<core::convert::Infallible> for $l
        where
            T: PortableEq,
        {
            fn portable_repr_eq(&self, other: &core::convert::Infallible) -> bool {
                let _ = other;
                true
            }
        }
    };
}

#[cfg(feature = "alloc")]
macro_rules! map_eq {
    ([$($lg:tt)*] $l:ty, [$($rg:tt)*] $r:ty) => {
        impl<$($lg)*, $($rg)*> PortableReprEq<$r> for $l
        where
            K1: PortableEq<K2>,
            V1: PortableEq<V2>,
        {
            fn portable_repr_eq(&self, other: &$r) -> bool {
                self.len() == other.len()
                    && self.iter().zip(other.iter()).all(|((lk, lv), (rk, rv))| {
                        lk.portable_eq(rk) && lv.portable_eq(rv)
                    })
            }
        }

        impl<$($lg)*, K2, V2> PortableReprEq<[(K2, V2)]> for $l
        where
            K1: PortableEq<K2>,
            V1: PortableEq<V2>,
        {
            fn portable_repr_eq(&self, other: &[(K2, V2)]) -> bool {
                self.len() == other.len()
                    && self.iter().zip(other.iter()).all(|((lk, lv), (rk, rv))| {
                        lk.portable_eq(rk) && lv.portable_eq(rv)
                    })
            }
        }

        impl<$($rg)*, K1, V1> PortableReprEq<$r> for [(K1, V1)]
        where
            K1: PortableEq<K2>,
            V1: PortableEq<V2>,
        {
            fn portable_repr_eq(&self, other: &$r) -> bool {
                self.len() == other.len()
                    && self.iter().zip(other.iter()).all(|((lk, lv), (rk, rv))| {
                        lk.portable_eq(rk) && lv.portable_eq(rv)
                    })
            }
        }

        impl<$($lg)*> PortableReprEq<core::convert::Infallible> for $l
        where
            K1: PortableEq,
            V1: PortableEq,
        {
            fn portable_repr_eq(&self, other: &core::convert::Infallible) -> bool {
                let _ = other;
                true
            }
        }
    };
}

cfg_select!(feature = "alloc" => {
    seq_eq!([T] alloc::collections::BTreeSet<T>, [U] alloc::collections::BTreeSet<U>);
    seq_eq!([T] alloc::collections::LinkedList<T>, [U] alloc::collections::LinkedList<U>);
    seq_eq!([T] alloc::collections::VecDeque<T>, [U] alloc::collections::VecDeque<U>);
    map_eq!(
        [K1, V1] alloc::collections::BTreeMap<K1, V1>,
        [K2, V2] alloc::collections::BTreeMap<K2, V2>
    );
} _ => {});

#[cfg(all(feature = "rkyv-0_8", feature = "alloc"))]
macro_rules! seq_eq_cross {
    ([$($lg:tt)*] $l:ty, [$($rg:tt)*] $r:ty) => {
        impl<$($lg)*, $($rg)*> PortableReprEq<$r> for $l
        where
            T: PortableEq<U>,
        {
            fn portable_repr_eq(&self, other: &$r) -> bool {
                self.len() == other.len()
                    && self.iter().zip(other.iter()).all(|(l, r)| l.portable_eq(r))
            }
        }

        impl<$($lg)*, $($rg)*> PortableReprEq<$l> for $r
        where
            U: PortableEq<T>,
        {
            fn portable_repr_eq(&self, other: &$l) -> bool {
                self.len() == other.len()
                    && self.iter().zip(other.iter()).all(|(l, r)| l.portable_eq(r))
            }
        }
    };
}

#[cfg(all(feature = "rkyv-0_8", feature = "alloc"))]
macro_rules! map_eq_cross {
    ([$($lg:tt)*] $l:ty, [$($rg:tt)*] $r:ty) => {
        impl<$($lg)*, $($rg)*> PortableReprEq<$r> for $l
        where
            K1: PortableEq<K2>,
            V1: PortableEq<V2>,
        {
            fn portable_repr_eq(&self, other: &$r) -> bool {
                self.len() == other.len()
                    && self.iter().zip(other.iter()).all(|((lk, lv), (rk, rv))| {
                        lk.portable_eq(rk) && lv.portable_eq(rv)
                    })
            }
        }

        impl<$($lg)*, $($rg)*> PortableReprEq<$l> for $r
        where
            K2: PortableEq<K1>,
            V2: PortableEq<V1>,
        {
            fn portable_repr_eq(&self, other: &$l) -> bool {
                self.len() == other.len()
                    && self.iter().zip(other.iter()).all(|((lk, lv), (rk, rv))| {
                        lk.portable_eq(rk) && lv.portable_eq(rv)
                    })
            }
        }
    };
}

cfg_select!(all(feature = "rkyv-0_8", feature = "alloc") => {
    seq_eq!(
        [T, const E: usize] rkyv_0_8::collections::btree_set::ArchivedBTreeSet<T, E>,
        [U, const F: usize] rkyv_0_8::collections::btree_set::ArchivedBTreeSet<U, F>
    );
    map_eq!(
        [K1, V1, const E: usize] rkyv_0_8::collections::btree_map::ArchivedBTreeMap<K1, V1, E>,
        [K2, V2, const F: usize] rkyv_0_8::collections::btree_map::ArchivedBTreeMap<K2, V2, F>
    );

    seq_eq_cross!(
        [T, const E: usize] rkyv_0_8::collections::btree_set::ArchivedBTreeSet<T, E>,
        [U] alloc::collections::BTreeSet<U>
    );
    map_eq_cross!(
        [K1, V1, const E: usize] rkyv_0_8::collections::btree_map::ArchivedBTreeMap<K1, V1, E>,
        [K2, V2] alloc::collections::BTreeMap<K2, V2>
    );
} _ => {});

macro_rules! tuple {
    ($r0:ident : ($t0:ident, $u0:ident $(,)?) $(, $rn:ident : ($tn:ident, $un:ident $(,)?))* $(,)?) => {
        tuple! { { $r0 ($t0 $u0) } { $(($rn $tn $un))* } }
    };
    ({ $r:ident $(($t:ident $u:ident))* } { ($r0:ident $t0:ident $u0:ident) $(($rn:ident $tn:ident $un:ident))* }) => {
        tuple! { { $r $(($t $u))* } { } }
        tuple! { { $r0 $(($t $u))* ($t0 $u0) } { $(($rn $tn $un))* } }
    };
    ({ $r:ident $(($t:ident $u:ident))* } { }) => {
        impl<$($t,)* $($u),*> PortableReprEq<repr::$r<$($u),*>> for repr::$r<$($t),*>
        where
            $($t: PortableEq<$u>,)*
        {
            fn portable_repr_eq(&self, other: &repr::$r<$($u),*>) -> bool {
                #[allow(non_snake_case)]
                fn eq<$($t,)* $($u),*>(
                    ($($t,)*): ($(&$t,)*),
                    ($($u,)*): ($(&$u,)*),
                ) -> bool
                where
                    $($t: PortableEq<$u>,)*
                {
                    $($t.portable_eq($u))&&*
                }

                eq(self.as_ref(), other.as_ref())
            }
        }

        impl<$($t),*> PortableReprEq<core::convert::Infallible> for repr::$r<$($t),*>
        where
            $($t: PortableEq,)*
        {
            fn portable_repr_eq(&self, other: &core::convert::Infallible) -> bool {
                let _ = other;
                true
            }
        }
    };
}

tuple! {
    Tuple1Repr: (T0, U0),
    Tuple2Repr: (T1, U1),
    Tuple3Repr: (T2, U2),
    Tuple4Repr: (T3, U3),
    Tuple5Repr: (T4, U4),
    Tuple6Repr: (T5, U5),
    Tuple7Repr: (T6, U6),
    Tuple8Repr: (T7, U7),
    Tuple9Repr: (T8, U8),
    Tuple10Repr: (T9, U9),
    Tuple11Repr: (T10, U10),
    Tuple12Repr: (T11, U11),
    Tuple13Repr: (T12, U12),
    Tuple14Repr: (T13, U13),
    Tuple15Repr: (T14, U14),
    Tuple16Repr: (T15, U15),
}

impl<T: ?Sized, U: ?Sized> PortableReprEq<core::marker::PhantomData<U>>
    for core::marker::PhantomData<T>
{
    fn portable_repr_eq(&self, other: &core::marker::PhantomData<U>) -> bool {
        let _ = other;
        true
    }
}

impl<T: ?Sized> PortableReprEq<core::convert::Infallible> for core::marker::PhantomData<T> {
    fn portable_repr_eq(&self, other: &core::convert::Infallible) -> bool {
        let _ = other;
        true
    }
}

impl PortableReprEq for core::marker::PhantomPinned {
    fn portable_repr_eq(&self, other: &Self) -> bool {
        let _ = other;
        true
    }
}

impl PortableReprEq<core::convert::Infallible> for core::marker::PhantomPinned {
    fn portable_repr_eq(&self, other: &core::convert::Infallible) -> bool {
        let _ = other;
        true
    }
}

impl<T, U> PortableReprEq<core::task::Poll<U>> for core::task::Poll<T>
where
    T: PortableEq<U>,
{
    fn portable_repr_eq(&self, other: &core::task::Poll<U>) -> bool {
        use core::task::Poll::{Pending, Ready};

        match (self, other) {
            (Ready(l), Ready(r)) => l.portable_eq(r),
            (Pending, Pending) => true,
            _ => false,
        }
    }
}

impl<T> PortableReprEq<core::convert::Infallible> for core::task::Poll<T>
where
    T: PortableEq,
{
    fn portable_repr_eq(&self, other: &core::convert::Infallible) -> bool {
        let _ = other;
        true
    }
}

impl<T, U> PortableReprEq<repr::OptionRepr<U>> for repr::OptionRepr<T>
where
    T: PortableEq<U> + ?Sized,
    U: ?Sized,
{
    fn portable_repr_eq(&self, other: &repr::OptionRepr<U>) -> bool {
        match (self.as_ref(), other.as_ref()) {
            (Some(l), Some(r)) => l.portable_eq(r),
            (None, None) => true,
            _ => false,
        }
    }
}

impl<T> PortableReprEq<core::convert::Infallible> for repr::OptionRepr<T>
where
    T: PortableEq + ?Sized,
{
    fn portable_repr_eq(&self, other: &core::convert::Infallible) -> bool {
        let _ = other;
        true
    }
}

impl<T1, E1, T2, E2> PortableReprEq<repr::ResultRepr<T2, E2>> for repr::ResultRepr<T1, E1>
where
    T1: PortableEq<T2> + ?Sized,
    E1: PortableEq<E2> + ?Sized,
    T2: ?Sized,
    E2: ?Sized,
{
    fn portable_repr_eq(&self, other: &repr::ResultRepr<T2, E2>) -> bool {
        match (self.as_ref(), other.as_ref()) {
            (Ok(l), Ok(r)) => l.portable_eq(r),
            (Err(l), Err(r)) => l.portable_eq(r),
            _ => false,
        }
    }
}

impl<T, E> PortableReprEq<core::convert::Infallible> for repr::ResultRepr<T, E>
where
    T: PortableEq + ?Sized,
    E: PortableEq + ?Sized,
{
    fn portable_repr_eq(&self, other: &core::convert::Infallible) -> bool {
        let _ = other;
        true
    }
}

impl<T, U> PortableReprEq<repr::BoundRepr<U>> for repr::BoundRepr<T>
where
    T: PortableEq<U> + ?Sized,
    U: ?Sized,
{
    fn portable_repr_eq(&self, other: &repr::BoundRepr<U>) -> bool {
        use core::ops::Bound::{Excluded, Included, Unbounded};

        match (self.as_ref(), other.as_ref()) {
            (Unbounded, Unbounded) => true,
            (Included(l), Included(r)) => l.portable_eq(r),
            (Excluded(l), Excluded(r)) => l.portable_eq(r),
            _ => false,
        }
    }
}

impl<T> PortableReprEq<core::convert::Infallible> for repr::BoundRepr<T>
where
    T: PortableEq + ?Sized,
{
    fn portable_repr_eq(&self, other: &core::convert::Infallible) -> bool {
        let _ = other;
        true
    }
}

#[cfg(feature = "triomphe-0_1")]
impl<A1, B1, A2, B2> PortableReprEq<repr::ArcUnionRepr<A2, B2>> for repr::ArcUnionRepr<A1, B1>
where
    A1: PortableEq<A2>,
    B1: PortableEq<B2>,
{
    fn portable_repr_eq(&self, other: &repr::ArcUnionRepr<A2, B2>) -> bool {
        use triomphe_0_1::ArcUnionBorrow::{First, Second};

        match (self.borrow(), other.borrow()) {
            (First(l), First(r)) => l.get().portable_eq(r.get()),
            (Second(l), Second(r)) => l.get().portable_eq(r.get()),
            _ => false,
        }
    }
}

#[cfg(feature = "triomphe-0_1")]
impl<A, B> PortableReprEq<core::convert::Infallible> for repr::ArcUnionRepr<A, B>
where
    A: PortableEq,
    B: PortableEq,
{
    fn portable_repr_eq(&self, other: &core::convert::Infallible) -> bool {
        let _ = other;
        true
    }
}

#[cfg(feature = "either-1")]
impl<L1, R1, L2, R2> PortableReprEq<either_1::Either<L2, R2>> for either_1::Either<L1, R1>
where
    L1: PortableEq<L2>,
    R1: PortableEq<R2>,
{
    fn portable_repr_eq(&self, other: &either_1::Either<L2, R2>) -> bool {
        use either_1::{Left, Right};

        match (self, other) {
            (Left(l), Left(r)) => l.portable_eq(r),
            (Right(l), Right(r)) => l.portable_eq(r),
            _ => false,
        }
    }
}

#[cfg(feature = "either-1")]
impl<L, R> PortableReprEq<core::convert::Infallible> for either_1::Either<L, R>
where
    L: PortableEq,
    R: PortableEq,
{
    fn portable_repr_eq(&self, other: &core::convert::Infallible) -> bool {
        let _ = other;
        true
    }
}
