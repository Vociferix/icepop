use super::Portable;

use core::ptr::NonNull;

pub trait VisitPortableRepr {
    type Repr: PortableRepr + ?Sized;

    fn visit_portable_repr<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Self::Repr) -> R;
}

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
        pub struct $n<$($t: ?Sized),*>($(NonNull<$t>),*);

        impl<$($t: ?Sized),*> $n<$($t),*> {
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

            pub fn as_ref(&self) -> ($(&$t,)*) {
                #[allow(non_snake_case)]
                fn make_ref<$($t: ?Sized),*>(
                    $n($($t),*): &$n<$($t),*>,
                ) -> ($(&$t,)*) {
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
map_repr!([T: VisitPortableRepr] |val: &core::cmp::Reverse<T>| -> &T { &val.0 });
self_repr!(core::cmp::Ordering);
map_repr!([P: core::ops::Deref<Target: VisitPortableRepr>] |val: &core::pin::Pin<P>| -> &P::Target { val });
self_repr!(core::task::Poll<T>);
self_repr!(core::time::Duration);

pub struct OptionRepr<T: ?Sized>(Option<NonNull<T>>);

impl<T: ?Sized> OptionRepr<T> {
    pub fn visit<F, R>(opt: Option<&T>, f: F) -> R
    where
        F: FnOnce(&Self) -> R,
    {
        f(&Self(opt.map(NonNull::from_ref)))
    }

    pub fn as_ref(&self) -> Option<&T> {
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

pub struct ResultRepr<T: ?Sized, E: ?Sized>(Result<NonNull<T>, NonNull<E>>);

impl<T: ?Sized, E: ?Sized> ResultRepr<T, E> {
    pub fn visit<F, R>(res: Result<&T, &E>, f: F) -> R
    where
        F: FnOnce(&Self) -> R,
    {
        f(&Self(res.map(NonNull::from_ref).map_err(NonNull::from_ref)))
    }

    pub fn as_ref(&self) -> Result<&T, &E> {
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

pub struct BoundRepr<T: ?Sized>(core::ops::Bound<NonNull<T>>);

impl<T: ?Sized> BoundRepr<T> {
    pub fn visit<F, R>(bound: core::ops::Bound<&T>, f: F) -> R
    where
        F: FnOnce(&Self) -> R,
    {
        f(&Self(bound.map(NonNull::from_ref)))
    }

    pub fn as_ref(&self) -> core::ops::Bound<&T> {
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

pub trait Sequence {
    type Item: Sized;

    type Iter<'a>: Iterator
    where
        Self: 'a,
        Self::Item: 'a;

    fn iter<'a>(&'a self) -> Self::Iter<'a>
    where
        Self::Item: 'a;

    fn is_empty<'a>(&'a self) -> bool
    where
        Self::Iter<'a>: ExactSizeIterator,
    {
        self.len() == 0
    }

    fn len<'a>(&'a self) -> usize
    where
        Self::Iter<'a>: ExactSizeIterator,
    {
        self.iter().len()
    }
}

impl<S, T> Sequence for S
where
    for<'a> &'a S: IntoIterator<Item = T>,
{
    type Item = T;

    type Iter<'a>
        = <&'a S as IntoIterator>::IntoIter
    where
        Self: 'a,
        T: 'a;

    fn iter<'a>(&'a self) -> Self::Iter<'a>
    where
        Self::Item: 'a,
    {
        self.into_iter()
    }
}

pub struct SequenceRepr<T: ?Sized>(NonNull<T>);

impl<T: ?Sized> SequenceRepr<T> {
    pub fn visit<F, R>(seq: &T, f: F) -> R
    where
        F: FnOnce(&Self) -> R,
    {
        f(&Self(NonNull::from_ref(seq)))
    }

    pub fn is_empty<'a>(&'a self) -> bool
    where
        &'a T: IntoIterator<IntoIter: ExactSizeIterator>,
    {
        self.len() == 0
    }

    pub fn len<'a>(&'a self) -> usize
    where
        &'a T: IntoIterator<IntoIter: ExactSizeIterator>,
    {
        self.iter().len()
    }

    pub fn iter<'a>(&'a self) -> <&'a T as IntoIterator>::IntoIter
    where
        &'a T: IntoIterator,
    {
        (&**self).into_iter()
    }
}

impl<T: ?Sized> core::ops::Deref for SequenceRepr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl<'a, T: ?Sized> IntoIterator for &'a SequenceRepr<T>
where
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;
    type IntoIter = <&'a T as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

self_repr!(SequenceRepr<T: ?Sized>);

cfg_select!(feature = "alloc" => {
    map_repr!([T: VisitPortableRepr + ?Sized] |val: &alloc::boxed::Box<T>| -> &T { val });
    map_repr!([T: VisitPortableRepr + ?Sized] |val: &alloc::rc::Rc<T>| -> &T { val });
    map_repr!([T: VisitPortableRepr + ?Sized] |val: &alloc::sync::Arc<T>| -> &T { val });
    map_repr!([T] |val: &alloc::vec::Vec<T>| -> &[T] { val });
    map_repr!(|val: &alloc::string::String| -> &str { val });
    map_repr!(|val: &alloc::ffi::CString| -> &core::ffi::CStr { val });
    map_repr!([T: VisitPortableRepr + alloc::borrow::ToOwned + ?Sized] |val: &alloc::borrow::Cow<'_, T>| -> &T { val });

    impl<T> VisitPortableRepr for alloc::collections::BTreeSet<T> {
        type Repr = SequenceRepr<Self>;

        fn visit_portable_repr<F, R>(&self, f: F) -> R
        where
            F: FnOnce(&Self::Repr) -> R,
        {
            SequenceRepr::<Self>::visit(self, f)
        }
    }

    impl<K, V> VisitPortableRepr for alloc::collections::BTreeMap<K, V> {
        type Repr = SequenceRepr<Self>;

        fn visit_portable_repr<F, R>(&self, f: F) -> R
        where
            F: FnOnce(&Self::Repr) -> R,
        {
            SequenceRepr::<Self>::visit(self, f)
        }
    }

    impl<T> VisitPortableRepr for alloc::collections::LinkedList<T> {
        type Repr = SequenceRepr<Self>;

        fn visit_portable_repr<F, R>(&self, f: F) -> R
        where
            F: FnOnce(&Self::Repr) -> R,
        {
            SequenceRepr::<Self>::visit(self, f)
        }
    }

    impl<T> VisitPortableRepr for alloc::collections::VecDeque<T> {
        type Repr = SequenceRepr<Self>;

        fn visit_portable_repr<F, R>(&self, f: F) -> R
        where
            F: FnOnce(&Self::Repr) -> R,
        {
            SequenceRepr::<Self>::visit(self, f)
        }
    }
} _ => {});

cfg_select!(feature = "rkyv-0_8" => {
    map_repr!([T: VisitPortableRepr + ?Sized] |val: &rkyv_0_8::seal::Seal<'_, T>| -> &T { val });

    map_repr! {
        [T: VisitPortableRepr + rkyv_0_8::traits::ArchivePointee + ?Sized]
        |val: &rkyv_0_8::boxed::ArchivedBox<T>| -> &T { val }
    }

    impl<T, const E: usize> VisitPortableRepr for rkyv_0_8::collections::btree_set::ArchivedBTreeSet<T, E> {
        type Repr = SequenceRepr<Self>;

        fn visit_portable_repr<F, R>(&self, f: F) -> R
        where
            F: FnOnce(&Self::Repr) -> R,
        {
            SequenceRepr::<Self>::visit(self, f)
        }
    }

    impl<K, V, const E: usize> VisitPortableRepr for rkyv_0_8::collections::btree_map::ArchivedBTreeMap<K, V, E> {
        type Repr = SequenceRepr<Self>;

        fn visit_portable_repr<F, R>(&self, f: F) -> R
        where
            F: FnOnce(&Self::Repr) -> R,
        {
            SequenceRepr::<Self>::visit(self, f)
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

    pub struct ArcUnionRepr<A, B>(ArcUnionReprInner<A, B>);

    enum ArcUnionReprInner<A, B> {
        First(NonNull<A>),
        Second(NonNull<B>),
    }

    impl<A, B> ArcUnionRepr<A, B> {
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

        pub fn borrow(&self) -> triomphe_0_1::ArcUnionBorrow<'_, A, B> {
            use triomphe_0_1::ArcUnionBorrow::{First, Second};

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
