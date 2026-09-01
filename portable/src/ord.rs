//! [`PortableOrd`], the platform-independent counterpart of [`Ord`].

use super::eq::{PortableEq, PortableReprEq};
use super::repr::{self, PortableRepr, VisitPortableRepr};

use core::cmp::Ordering;

/// Ordering that gives the same answer on every platform, across types.
///
/// Blanket-implemented for every [`VisitPortableRepr`] type whose representation implements
/// [`PortableReprOrd`] for the other type's representation, so any two types sharing a
/// representation can be ordered against each other.
///
/// # Example
///
/// ```
/// use portable::PortableOrd;
///
/// let slice: &[u32] = &[1, 2];
///
/// assert!(2usize.portable_cmp(&10u64).is_lt());
/// assert!([1u32, 2, 3].portable_cmp(slice).is_gt());
/// assert!(Some(1usize).portable_cmp(&None::<u64>).is_gt());
/// ```
pub trait PortableOrd<K: ?Sized = Self>: PortableEq<K> {
    /// Returns the ordering of this value relative to `other`.
    ///
    /// # Example
    ///
    /// ```
    /// use core::cmp::Ordering;
    /// use portable::PortableOrd;
    ///
    /// assert_eq!(2usize.portable_cmp(&10u64), Ordering::Less);
    /// assert_eq!(2usize.portable_cmp(&2u64), Ordering::Equal);
    /// ```
    fn portable_cmp(&self, other: &K) -> Ordering;
}

/// Ordering between two [portable representations](PortableRepr).
///
/// This is the trait to implement for a representation type; [`PortableOrd`] on the types that
/// share those representations follows from it. Implementing it for
/// [`Infallible`](core::convert::Infallible) as well makes a representation orderable against
/// representations that can never hold a value, such as the bounds of a `RangeFull`.
///
/// # Example
///
/// ```
/// use portable::ord::PortableReprOrd;
///
/// // `usize` orders against every width it may be stored as.
/// assert!(2usize.portable_repr_cmp(&10u64).is_lt());
/// ```
pub trait PortableReprOrd<K: PortableRepr + ?Sized = Self>: PortableReprEq<K> {
    /// Returns the ordering of this representation relative to `other`.
    ///
    /// # Example
    ///
    /// ```
    /// use core::cmp::Ordering;
    /// use portable::ord::PortableReprOrd;
    ///
    /// assert_eq!(2usize.portable_repr_cmp(&10u64), Ordering::Less);
    /// assert_eq!(2usize.portable_repr_cmp(&2u64), Ordering::Equal);
    /// ```
    fn portable_repr_cmp(&self, other: &K) -> Ordering;
}

impl<T, U> PartialOrd<super::Portable<U>> for super::Portable<T>
where
    T: PortableOrd<U> + ?Sized,
    U: ?Sized,
{
    fn partial_cmp(&self, other: &super::Portable<U>) -> Option<Ordering> {
        Some(self.0.portable_cmp(&other.0))
    }
}

impl<T> Ord for super::Portable<T>
where
    T: PortableOrd + ?Sized,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.portable_cmp(&other.0)
    }
}

impl<T, U> PortableOrd<U> for T
where
    T: VisitPortableRepr + ?Sized,
    U: VisitPortableRepr + ?Sized,
    T::Repr: PortableRepr + PortableReprOrd<U::Repr>,
    U::Repr: PortableRepr,
{
    fn portable_cmp(&self, other: &U) -> Ordering {
        self.visit_portable_repr(move |l| {
            other.visit_portable_repr(move |r| l.portable_repr_cmp(r))
        })
    }
}

impl<K: PortableOrd + PortableRepr + ?Sized> PortableReprOrd<K> for core::convert::Infallible {
    fn portable_repr_cmp(&self, other: &K) -> Ordering {
        let _ = other;
        Ordering::Equal
    }
}

impl<T> PortableReprOrd for crate::AssertPortable<T>
where
    T: Ord + ?Sized,
{
    fn portable_repr_cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&self.0, &other.0)
    }
}

impl<T> PortableReprOrd<core::convert::Infallible> for crate::AssertPortable<T>
where
    T: Ord + ?Sized,
{
    fn portable_repr_cmp(&self, other: &core::convert::Infallible) -> Ordering {
        let _ = other;
        Ordering::Equal
    }
}

macro_rules! cmp_self {
    ($t:ty) => {
        impl PortableReprOrd for $t {
            #[inline]
            fn portable_repr_cmp(&self, other: &Self) -> Ordering {
                Ord::cmp(self, other)
            }
        }

        impl PortableReprOrd<core::convert::Infallible> for $t {
            #[inline]
            fn portable_repr_cmp(&self, other: &core::convert::Infallible) -> Ordering {
                let _ = other;
                Ordering::Equal
            }
        }
    };
}

cmp_self!(());
cmp_self!(bool);
cmp_self!(char);
cmp_self!(u8);
cmp_self!(u16);
cmp_self!(u32);
cmp_self!(u64);
cmp_self!(u128);
cmp_self!(usize);
cmp_self!(i8);
cmp_self!(i16);
cmp_self!(i32);
cmp_self!(i64);
cmp_self!(i128);
cmp_self!(isize);
cmp_self!(str);
cmp_self!(core::cmp::Ordering);
cmp_self!(core::ffi::CStr);
cmp_self!(core::net::IpAddr);
cmp_self!(core::net::SocketAddr);
cmp_self!(core::time::Duration);

macro_rules! size_types {
    ($($t:ident [$r:ident] { $($i:ident),* $(,)? }),* $(,)?) => {
        $($(
            impl PortableReprOrd<$i> for $t {
                #[inline]
                fn portable_repr_cmp(&self, other: &$i) -> Ordering {
                    Ord::cmp(&(*self as $r), &(*other as $r))
                }
            }

            impl PortableReprOrd<$t> for $i {
                #[inline]
                fn portable_repr_cmp(&self, other: &$t) -> Ordering {
                    Ord::cmp(&(*self as $r), &(*other as $r))
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

impl<T, U> PortableReprOrd<[U]> for [T]
where
    T: PortableOrd<U>,
{
    fn portable_repr_cmp(&self, other: &[U]) -> Ordering {
        let mut l = self.iter();
        let mut r = other.iter();
        loop {
            match (l.next(), r.next()) {
                (Some(l), Some(r)) => match l.portable_cmp(r) {
                    Ordering::Equal => {}
                    ord => return ord,
                },
                (None, None) => return Ordering::Equal,
                (Some(_), None) => return Ordering::Greater,
                (None, Some(_)) => return Ordering::Less,
            }
        }
    }
}

impl<T> PortableReprOrd<core::convert::Infallible> for [T]
where
    T: PortableOrd,
{
    fn portable_repr_cmp(&self, other: &core::convert::Infallible) -> Ordering {
        let _ = other;
        Ordering::Equal
    }
}

#[cfg(feature = "alloc")]
macro_rules! seq_cmp {
    ([$($lg:tt)*] $l:ty, [$($rg:tt)*] $r:ty) => {
        impl<$($lg)*, $($rg)*> PortableReprOrd<$r> for $l
        where
            T: PortableOrd<U>,
        {
            fn portable_repr_cmp(&self, other: &$r) -> Ordering {
                seq_cmp(self.iter(), other.iter())
            }
        }

        impl<$($lg)*, U> PortableReprOrd<[U]> for $l
        where
            T: PortableOrd<U>,
        {
            fn portable_repr_cmp(&self, other: &[U]) -> Ordering {
                seq_cmp(self.iter(), other.iter())
            }
        }

        impl<$($rg)*, T> PortableReprOrd<$r> for [T]
        where
            T: PortableOrd<U>,
        {
            fn portable_repr_cmp(&self, other: &$r) -> Ordering {
                seq_cmp(self.iter(), other.iter())
            }
        }

        impl<$($lg)*> PortableReprOrd<core::convert::Infallible> for $l
        where
            T: PortableOrd,
        {
            fn portable_repr_cmp(&self, other: &core::convert::Infallible) -> Ordering {
                let _ = other;
                Ordering::Equal
            }
        }
    };
}

#[cfg(feature = "alloc")]
macro_rules! map_cmp {
    ([$($lg:tt)*] $l:ty, [$($rg:tt)*] $r:ty) => {
        impl<$($lg)*, $($rg)*> PortableReprOrd<$r> for $l
        where
            K1: PortableOrd<K2>,
            V1: PortableOrd<V2>,
        {
            fn portable_repr_cmp(&self, other: &$r) -> Ordering {
                map_cmp(self.iter(), other.iter())
            }
        }

        impl<$($lg)*, K2, V2> PortableReprOrd<[(K2, V2)]> for $l
        where
            K1: PortableOrd<K2>,
            V1: PortableOrd<V2>,
        {
            fn portable_repr_cmp(&self, other: &[(K2, V2)]) -> Ordering {
                map_cmp(self.iter(), other.iter().map(|(k, v)| (k, v)))
            }
        }

        impl<$($rg)*, K1, V1> PortableReprOrd<$r> for [(K1, V1)]
        where
            K1: PortableOrd<K2>,
            V1: PortableOrd<V2>,
        {
            fn portable_repr_cmp(&self, other: &$r) -> Ordering {
                map_cmp(self.iter().map(|(k, v)| (k, v)), other.iter())
            }
        }

        impl<$($lg)*> PortableReprOrd<core::convert::Infallible> for $l
        where
            K1: PortableOrd,
            V1: PortableOrd,
        {
            fn portable_repr_cmp(&self, other: &core::convert::Infallible) -> Ordering {
                let _ = other;
                Ordering::Equal
            }
        }
    };
}

#[cfg(feature = "alloc")]
fn seq_cmp<'l, 'r, L, R, T, U>(mut left: L, mut right: R) -> Ordering
where
    L: Iterator<Item = &'l T>,
    R: Iterator<Item = &'r U>,
    T: PortableOrd<U> + 'l,
    U: 'r,
{
    loop {
        match (left.next(), right.next()) {
            (Some(l), Some(r)) => match l.portable_cmp(r) {
                Ordering::Equal => {}
                ord => return ord,
            },
            (None, None) => return Ordering::Equal,
            (Some(_), None) => return Ordering::Greater,
            (None, Some(_)) => return Ordering::Less,
        }
    }
}

#[cfg(feature = "alloc")]
fn map_cmp<'l, 'r, L, R, K1, V1, K2, V2>(mut left: L, mut right: R) -> Ordering
where
    L: Iterator<Item = (&'l K1, &'l V1)>,
    R: Iterator<Item = (&'r K2, &'r V2)>,
    K1: PortableOrd<K2> + 'l,
    V1: PortableOrd<V2> + 'l,
    K2: 'r,
    V2: 'r,
{
    loop {
        match (left.next(), right.next()) {
            (Some((lk, lv)), Some((rk, rv))) => match lk.portable_cmp(rk) {
                Ordering::Equal => match lv.portable_cmp(rv) {
                    Ordering::Equal => {}
                    ord => return ord,
                },
                ord => return ord,
            },
            (None, None) => return Ordering::Equal,
            (Some(_), None) => return Ordering::Greater,
            (None, Some(_)) => return Ordering::Less,
        }
    }
}

cfg_select!(feature = "alloc" => {
    seq_cmp!([T] alloc::collections::BTreeSet<T>, [U] alloc::collections::BTreeSet<U>);
    seq_cmp!([T] alloc::collections::LinkedList<T>, [U] alloc::collections::LinkedList<U>);
    seq_cmp!([T] alloc::collections::VecDeque<T>, [U] alloc::collections::VecDeque<U>);
    map_cmp!(
        [K1, V1] alloc::collections::BTreeMap<K1, V1>,
        [K2, V2] alloc::collections::BTreeMap<K2, V2>
    );
} _ => {});

#[cfg(all(feature = "rkyv-0_8", feature = "alloc"))]
macro_rules! seq_cmp_cross {
    ([$($lg:tt)*] $l:ty, [$($rg:tt)*] $r:ty) => {
        impl<$($lg)*, $($rg)*> PortableReprOrd<$r> for $l
        where
            T: PortableOrd<U>,
        {
            fn portable_repr_cmp(&self, other: &$r) -> Ordering {
                seq_cmp(self.iter(), other.iter())
            }
        }

        impl<$($lg)*, $($rg)*> PortableReprOrd<$l> for $r
        where
            U: PortableOrd<T>,
        {
            fn portable_repr_cmp(&self, other: &$l) -> Ordering {
                seq_cmp(self.iter(), other.iter())
            }
        }
    };
}

#[cfg(all(feature = "rkyv-0_8", feature = "alloc"))]
macro_rules! map_cmp_cross {
    ([$($lg:tt)*] $l:ty, [$($rg:tt)*] $r:ty) => {
        impl<$($lg)*, $($rg)*> PortableReprOrd<$r> for $l
        where
            K1: PortableOrd<K2>,
            V1: PortableOrd<V2>,
        {
            fn portable_repr_cmp(&self, other: &$r) -> Ordering {
                map_cmp(self.iter(), other.iter())
            }
        }

        impl<$($lg)*, $($rg)*> PortableReprOrd<$l> for $r
        where
            K2: PortableOrd<K1>,
            V2: PortableOrd<V1>,
        {
            fn portable_repr_cmp(&self, other: &$l) -> Ordering {
                map_cmp(self.iter(), other.iter())
            }
        }
    };
}

cfg_select!(all(feature = "rkyv-0_8", feature = "alloc") => {
    seq_cmp!(
        [T, const E: usize] rkyv_0_8::collections::btree_set::ArchivedBTreeSet<T, E>,
        [U, const F: usize] rkyv_0_8::collections::btree_set::ArchivedBTreeSet<U, F>
    );
    map_cmp!(
        [K1, V1, const E: usize] rkyv_0_8::collections::btree_map::ArchivedBTreeMap<K1, V1, E>,
        [K2, V2, const F: usize] rkyv_0_8::collections::btree_map::ArchivedBTreeMap<K2, V2, F>
    );

    seq_cmp_cross!(
        [T, const E: usize] rkyv_0_8::collections::btree_set::ArchivedBTreeSet<T, E>,
        [U] alloc::collections::BTreeSet<U>
    );
    map_cmp_cross!(
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
        impl<$($t,)* $($u),*> PortableReprOrd<repr::$r<$($u),*>> for repr::$r<$($t),*>
        where
            $($t: PortableOrd<$u>,)*
        {
            fn portable_repr_cmp(&self, other: &repr::$r<$($u),*>) -> Ordering {
                #[allow(non_snake_case)]
                fn cmp<$($t,)* $($u),*>(
                    ($($t,)*): ($(&$t,)*),
                    ($($u,)*): ($(&$u,)*),
                ) -> Ordering
                where
                    $($t: PortableOrd<$u>,)*
                {
                    $(match $t.portable_cmp($u) {
                        Ordering::Equal => {},
                        ord => return ord,
                    })*

                    Ordering::Equal
                }

                cmp(self.as_ref(), other.as_ref())
            }
        }

        impl<$($t),*> PortableReprOrd<core::convert::Infallible> for repr::$r<$($t),*>
        where
            $($t: PortableOrd,)*
        {
            fn portable_repr_cmp(&self, other: &core::convert::Infallible) -> Ordering {
                let _ = other;
                Ordering::Equal
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

impl<T: ?Sized, U: ?Sized> PortableReprOrd<core::marker::PhantomData<U>>
    for core::marker::PhantomData<T>
{
    fn portable_repr_cmp(&self, other: &core::marker::PhantomData<U>) -> Ordering {
        let _ = other;
        Ordering::Equal
    }
}

impl<T: PortableOrd + ?Sized> PortableReprOrd<core::convert::Infallible>
    for core::marker::PhantomData<T>
{
    fn portable_repr_cmp(&self, other: &core::convert::Infallible) -> Ordering {
        let _ = other;
        Ordering::Equal
    }
}

impl PortableReprOrd for core::marker::PhantomPinned {
    fn portable_repr_cmp(&self, other: &Self) -> Ordering {
        let _ = other;
        Ordering::Equal
    }
}

impl PortableReprOrd<core::convert::Infallible> for core::marker::PhantomPinned {
    fn portable_repr_cmp(&self, other: &core::convert::Infallible) -> Ordering {
        let _ = other;
        Ordering::Equal
    }
}

impl<T, U> PortableReprOrd<core::task::Poll<U>> for core::task::Poll<T>
where
    T: PortableOrd<U>,
{
    fn portable_repr_cmp(&self, other: &core::task::Poll<U>) -> Ordering {
        use core::task::Poll::{Pending, Ready};

        match (self, other) {
            (Ready(l), Ready(r)) => l.portable_cmp(r),
            (Pending, Pending) => Ordering::Equal,
            (Ready(_), Pending) => Ordering::Less,
            (Pending, Ready(_)) => Ordering::Greater,
        }
    }
}

impl<T> PortableReprOrd<core::convert::Infallible> for core::task::Poll<T>
where
    T: PortableOrd,
{
    fn portable_repr_cmp(&self, other: &core::convert::Infallible) -> Ordering {
        let _ = other;
        Ordering::Equal
    }
}

impl<T, U> PortableReprOrd<repr::OptionRepr<U>> for repr::OptionRepr<T>
where
    T: PortableOrd<U>,
{
    fn portable_repr_cmp(&self, other: &repr::OptionRepr<U>) -> Ordering {
        match (self.as_ref(), other.as_ref()) {
            (Some(l), Some(r)) => l.portable_cmp(r),
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
        }
    }
}

impl<T> PortableReprOrd<core::convert::Infallible> for repr::OptionRepr<T>
where
    T: PortableOrd + ?Sized,
{
    fn portable_repr_cmp(&self, other: &core::convert::Infallible) -> Ordering {
        let _ = other;
        Ordering::Equal
    }
}

impl<T1, E1, T2, E2> PortableReprOrd<repr::ResultRepr<T2, E2>> for repr::ResultRepr<T1, E1>
where
    T1: PortableOrd<T2> + ?Sized,
    E1: PortableOrd<E2> + ?Sized,
    T2: ?Sized,
    E2: ?Sized,
{
    fn portable_repr_cmp(&self, other: &repr::ResultRepr<T2, E2>) -> Ordering {
        match (self.as_ref(), other.as_ref()) {
            (Ok(l), Ok(r)) => l.portable_cmp(r),
            (Err(l), Err(r)) => l.portable_cmp(r),
            (Ok(_), Err(_)) => Ordering::Less,
            (Err(_), Ok(_)) => Ordering::Greater,
        }
    }
}

impl<T, E> PortableReprOrd<core::convert::Infallible> for repr::ResultRepr<T, E>
where
    T: PortableOrd + ?Sized,
    E: PortableOrd + ?Sized,
{
    fn portable_repr_cmp(&self, other: &core::convert::Infallible) -> Ordering {
        let _ = other;
        Ordering::Equal
    }
}

#[cfg(feature = "triomphe-0_1")]
impl<A1, B1, A2, B2> PortableReprOrd<repr::ArcUnionRepr<A2, B2>> for repr::ArcUnionRepr<A1, B1>
where
    A1: PortableOrd<A2>,
    B1: PortableOrd<B2>,
{
    fn portable_repr_cmp(&self, other: &repr::ArcUnionRepr<A2, B2>) -> Ordering {
        use triomphe_0_1::ArcUnionBorrow::{First, Second};

        match (self.borrow(), other.borrow()) {
            (First(l), First(r)) => l.get().portable_cmp(r.get()),
            (Second(l), Second(r)) => l.get().portable_cmp(r.get()),
            (First(_), _) => Ordering::Less,
            (_, First(_)) => Ordering::Greater,
        }
    }
}

#[cfg(feature = "triomphe-0_1")]
impl<A, B> PortableReprOrd<core::convert::Infallible> for repr::ArcUnionRepr<A, B>
where
    A: PortableOrd,
    B: PortableOrd,
{
    fn portable_repr_cmp(&self, other: &core::convert::Infallible) -> Ordering {
        let _ = other;
        Ordering::Equal
    }
}

#[cfg(feature = "either-1")]
impl<L1, R1, L2, R2> PortableReprOrd<either_1::Either<L2, R2>> for either_1::Either<L1, R1>
where
    L1: PortableOrd<L2>,
    R1: PortableOrd<R2>,
{
    fn portable_repr_cmp(&self, other: &either_1::Either<L2, R2>) -> Ordering {
        use either_1::{Left, Right};

        match (self, other) {
            (Left(l), Left(r)) => l.portable_cmp(r),
            (Right(l), Right(r)) => l.portable_cmp(r),
            (Left(_), _) => Ordering::Less,
            (_, Left(_)) => Ordering::Greater,
        }
    }
}

#[cfg(feature = "either-1")]
impl<L, R> PortableReprOrd<core::convert::Infallible> for either_1::Either<L, R>
where
    L: PortableOrd,
    R: PortableOrd,
{
    fn portable_repr_cmp(&self, other: &core::convert::Infallible) -> Ordering {
        let _ = other;
        Ordering::Equal
    }
}
