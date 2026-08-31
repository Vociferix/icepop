#![cfg_attr(not(feature = "std"), no_std)]
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

#[derive(Debug, Clone, Copy, Default)]
#[repr(transparent)]
pub struct Portable<T: ?Sized>(pub T);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct AssertPortable<T: ?Sized>(pub T);

impl<T: ?Sized> Portable<T> {
    pub const fn from_ref(value: &T) -> &Self {
        unsafe { &*(value as *const T as *const Self) }
    }

    pub const fn from_mut(value: &mut T) -> &mut Self {
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
    pub const fn from_ref(value: &T) -> &Self {
        unsafe { &*(value as *const T as *const Self) }
    }

    pub const fn from_mut(value: &mut T) -> &mut Self {
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
