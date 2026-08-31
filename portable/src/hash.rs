use core::hash::Hasher;
use core::ops::RangeBounds;

pub trait PortableHash {
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher;

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

unsafe fn as_bytes<T>(val: &T) -> &[u8] {
    unsafe {
        core::slice::from_raw_parts(val as *const T as *const u8, core::mem::size_of_val(val))
    }
}

unsafe fn slice_as_bytes<T>(slice: &[T]) -> &[u8] {
    unsafe { core::slice::from_raw_parts(slice.as_ptr().cast(), core::mem::size_of_val(slice)) }
}

impl<T> PortableHash for &T
where
    T: PortableHash,
{
    fn portable_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        T::portable_hash(*self, state);
    }
}

impl<T> PortableHash for &mut T
where
    T: PortableHash,
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
            Self::V4(ip) => (false, ip).portable_hash(state),
            Self::V6(ip) => (true, ip).portable_hash(state),
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
            state.write(unsafe { as_bytes(self) })
        }

        fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
        where
            H: Hasher,
        {
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
            state.write(unsafe { as_bytes(self) });
        }

        fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
        where
            H: Hasher,
        {
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
            state.write(unsafe { as_bytes(self) });
        }

        fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
        where
            H: Hasher,
        {
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
            state.write(unsafe { as_bytes(self) });
        }

        fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
        where
            H: Hasher,
        {
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
            state.write(unsafe { as_bytes(self) });
        }

        fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
        where
            H: Hasher,
        {
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
            state.write(unsafe { as_bytes(self) });
        }

        fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
        where
            H: Hasher,
        {
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
            state.write(unsafe { as_bytes(self) });
        }

        fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
        where
            H: Hasher,
        {
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
            state.write(unsafe { as_bytes(self) });
        }

        fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
        where
            H: Hasher,
        {
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
            state.write(unsafe { as_bytes(self) });
        }

        fn portable_hash_slice<H>(slice: &[Self], state: &mut H)
        where
            H: Hasher,
        {
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

    impl<T, const E: usize> PortableHash for rkyv_0_8::collections::btree_set::ArchivedBTreeSet<T, E>
    where
        T: PortableHash,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.iter().for_each(|item| item.portable_hash(state));
        }
    }

    impl<K, V, const E: usize> PortableHash for rkyv_0_8::collections::btree_map::ArchivedBTreeMap<K, V, E>
    where
        K: PortableHash,
        V: PortableHash,
    {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
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
        }
    }

    impl PortableHash for rkyv_0_8::net::ArchivedSocketAddr {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            (self.ip(), self.port()).portable_hash(state);
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
            (*self as u8).portable_hash(state);
        }
    }

    impl PortableHash for ascii_1::AsciiStr {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_bytes().portable_hash(state);
        }
    }

    #[cfg(feature = "alloc")]
    impl PortableHash for ascii_1::AsciiString {
        fn portable_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_bytes().portable_hash(state);
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
