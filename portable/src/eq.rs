use super::repr::{self, PortableRepr, VisitPortableRepr};

pub trait PortableEq<K: ?Sized = Self> {
    fn portable_eq(&self, other: &K) -> bool;
}

pub trait PortableReprEq<K: PortableRepr + ?Sized = Self>: PortableRepr {
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

impl<T: ?Sized, U: ?Sized> PortableReprEq<repr::SequenceRepr<U>> for repr::SequenceRepr<T>
where
    for<'a> &'a T:
        IntoIterator<Item: PortableEq<<&'a U as IntoIterator>::Item>, IntoIter: ExactSizeIterator>,
    for<'a> &'a U: IntoIterator<IntoIter: ExactSizeIterator>,
{
    fn portable_repr_eq(&self, other: &repr::SequenceRepr<U>) -> bool {
        self.len() == other.len() && self.iter().zip(other).all(|(l, r)| l.portable_eq(&r))
    }
}

impl<T: ?Sized> PortableReprEq<core::convert::Infallible> for repr::SequenceRepr<T>
where
    for<'a> &'a T: IntoIterator<Item: PortableEq, IntoIter: ExactSizeIterator>,
{
    fn portable_repr_eq(&self, other: &core::convert::Infallible) -> bool {
        let _ = other;
        true
    }
}

impl<T, U: ?Sized> PortableReprEq<repr::SequenceRepr<U>> for [T]
where
    for<'a> &'a T: PortableEq<<&'a U as IntoIterator>::Item>,
    for<'a> &'a U: IntoIterator<IntoIter: ExactSizeIterator>,
{
    fn portable_repr_eq(&self, other: &repr::SequenceRepr<U>) -> bool {
        self.len() == other.len() && self.iter().zip(other).all(|(l, r)| l.portable_eq(&r))
    }
}

impl<T: ?Sized, U> PortableReprEq<[U]> for repr::SequenceRepr<T>
where
    for<'a> &'a T: IntoIterator<Item: PortableEq<&'a U>, IntoIter: ExactSizeIterator>,
{
    fn portable_repr_eq(&self, other: &[U]) -> bool {
        self.len() == other.len() && self.iter().zip(other).all(|(l, r)| l.portable_eq(&r))
    }
}

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
