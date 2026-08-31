use core::hash::{BuildHasher, Hasher};

use crate::hash::PortableHash;

pub trait PortableHasher: Hasher {}

pub trait PortableBuildHasher: BuildHasher<Hasher: PortableHasher> {
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
