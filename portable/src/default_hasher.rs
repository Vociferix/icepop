use rapidhash::quality::{RapidHasher, SeedableState};

use core::hash::{BuildHasher, Hasher};

use crate::hash::PortableHash;
use crate::hasher::PortableHasher;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "rkyv-0_8",
    derive(rkyv_0_8::Archive, rkyv_0_8::Deserialize, rkyv_0_8::Serialize)
)]
#[cfg_attr(
    feature = "rkyv-0_8",
    rkyv(derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash))
)]
#[cfg_attr(feature = "rkyv-0_8", rkyv(crate = rkyv_0_8))]
pub struct DefaultHasherSeed {
    #[cfg_attr(feature = "rkyv-0_8", rkyv(omit_bounds))]
    seed: u64,
}

#[derive(Clone)]
pub struct DefaultHasher {
    hasher: RapidHasher<'static>,
}

fn default_seed_raw() -> u64 {
    use core::sync::atomic::{AtomicU64, Ordering};

    static RAW_SEED: AtomicU64 = AtomicU64::new(const_random::const_random!(u64));

    #[cfg(feature = "getrandom")]
    {
        use core::sync::atomic::AtomicU8;

        static GUARD: AtomicU8 = AtomicU8::new(0);

        loop {
            match GUARD.compare_exchange(0, 0xff, Ordering::Acquire, Ordering::Relaxed) {
                Ok(_) => {
                    struct Guard;

                    impl Drop for Guard {
                        fn drop(&mut self) {
                            GUARD.store(0, Ordering::Release);
                        }
                    }

                    let guard = Guard;
                    let seed = getrandom::u64().expect("getrandom failed");
                    core::mem::forget(guard);

                    RAW_SEED.store(seed + 1, Ordering::Relaxed);
                    GUARD.store(1, Ordering::Release);

                    return seed;
                }
                Err(1) => {
                    break;
                }
                _ => {}
            }
        }
    }

    RAW_SEED.fetch_add(1, Ordering::Relaxed)
}

impl Hasher for DefaultHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.hasher.finish()
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.hasher.write(bytes);
    }

    #[inline]
    fn write_u8(&mut self, v: u8) {
        v.portable_hash(&mut self.hasher);
    }

    #[inline]
    fn write_u16(&mut self, v: u16) {
        v.portable_hash(&mut self.hasher);
    }

    #[inline]
    fn write_u32(&mut self, v: u32) {
        v.portable_hash(&mut self.hasher);
    }

    #[inline]
    fn write_u64(&mut self, v: u64) {
        v.portable_hash(&mut self.hasher);
    }

    #[inline]
    fn write_u128(&mut self, v: u128) {
        v.portable_hash(&mut self.hasher);
    }

    #[inline]
    fn write_usize(&mut self, v: usize) {
        v.portable_hash(&mut self.hasher);
    }

    #[inline]
    fn write_i8(&mut self, v: i8) {
        v.portable_hash(&mut self.hasher);
    }

    #[inline]
    fn write_i16(&mut self, v: i16) {
        v.portable_hash(&mut self.hasher);
    }

    #[inline]
    fn write_i32(&mut self, v: i32) {
        v.portable_hash(&mut self.hasher);
    }

    #[inline]
    fn write_i64(&mut self, v: i64) {
        v.portable_hash(&mut self.hasher);
    }

    #[inline]
    fn write_i128(&mut self, v: i128) {
        v.portable_hash(&mut self.hasher);
    }

    #[inline]
    fn write_isize(&mut self, v: isize) {
        v.portable_hash(&mut self.hasher);
    }
}

impl PortableHasher for DefaultHasher {}

impl DefaultHasherSeed {
    #[inline]
    pub fn new() -> Self {
        let secrets = rapidhash::v3::RapidSecrets {
            seed: default_seed_raw(),
            ..rapidhash::v3::DEFAULT_RAPID_SECRETS
        };
        Self::with_seed(rapidhash::v3::rapidhash_v3_inline::<true, false, false>(
            &secrets.seed.to_ne_bytes(),
            &secrets,
        ))
    }

    #[inline]
    pub const fn with_seed(seed: u64) -> Self {
        Self { seed }
    }

    #[inline]
    pub const fn seed(&self) -> u64 {
        self.seed
    }
}

impl Default for DefaultHasherSeed {
    fn default() -> Self {
        Self::new()
    }
}

impl BuildHasher for DefaultHasherSeed {
    type Hasher = DefaultHasher;

    fn build_hasher(&self) -> Self::Hasher {
        DefaultHasher {
            hasher: SeedableState::custom(self.seed, &rapidhash::v3::DEFAULT_RAPID_SECRETS.secrets)
                .build_hasher(),
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for DefaultHasherSeed {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut ser = serializer.serialize_struct("DefaultHasherSeed", 1)?;
        ser.serialize_field("seed", &self.seed)?;
        ser.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for DefaultHasherSeed {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        struct Field;

        impl<'de> serde::Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct Visitor;

                impl<'de> serde::de::Visitor<'de> for Visitor {
                    type Value = Field;

                    fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                        f.write_str("\"seed\"")
                    }

                    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        if v != 0 {
                            return Err(E::invalid_value(serde::de::Unexpected::Signed(v), &"0"));
                        }

                        Ok(Field)
                    }

                    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        if v != 0 {
                            return Err(E::invalid_value(serde::de::Unexpected::Unsigned(v), &"0"));
                        }

                        Ok(Field)
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        if !v.eq_ignore_ascii_case("seed") {
                            return Err(E::invalid_value(serde::de::Unexpected::Str(v), &self));
                        }

                        Ok(Field)
                    }
                }

                deserializer.deserialize_identifier(Visitor)
            }
        }

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = DefaultHasherSeed;

            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("a hasher seed")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let seed: u64 = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::missing_field("seed"))?;

                Ok(DefaultHasherSeed { seed })
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                if map.next_key::<Field>()?.is_none() {
                    return Err(serde::de::Error::missing_field("seed"));
                }

                let seed: u64 = map.next_value()?;

                if map.next_key::<Field>()?.is_some() {
                    return Err(serde::de::Error::duplicate_field("seed"));
                }

                Ok(DefaultHasherSeed { seed })
            }
        }

        deserializer.deserialize_struct("DefaultHasherSeed", &["seed"], Visitor)
    }
}

#[cfg(feature = "rkyv-0_8")]
impl ArchivedDefaultHasherSeed {
    #[inline]
    pub fn new() -> Self {
        Self {
            seed: rkyv_0_8::rend::u64_le::from_native(DefaultHasherSeed::new().seed),
        }
    }

    #[inline]
    pub const fn with_seed(seed: u64) -> Self {
        Self {
            seed: rkyv_0_8::rend::u64_le::from_native(seed),
        }
    }

    #[inline]
    pub const fn seed(&self) -> u64 {
        self.seed.to_native()
    }
}

#[cfg(feature = "rkyv-0_8")]
impl Default for ArchivedDefaultHasherSeed {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "rkyv-0_8")]
impl BuildHasher for ArchivedDefaultHasherSeed {
    type Hasher = DefaultHasher;

    fn build_hasher(&self) -> Self::Hasher {
        DefaultHasher {
            hasher: SeedableState::custom(
                self.seed.to_native(),
                &rapidhash::v3::DEFAULT_RAPID_SECRETS.secrets,
            )
            .build_hasher(),
        }
    }
}

#[cfg(feature = "rkyv-0_8")]
impl From<ArchivedDefaultHasherSeed> for DefaultHasherSeed {
    fn from(archive: ArchivedDefaultHasherSeed) -> Self {
        Self {
            seed: archive.seed.to_native(),
        }
    }
}

#[cfg(feature = "rkyv-0_8")]
impl From<DefaultHasherSeed> for ArchivedDefaultHasherSeed {
    fn from(seed: DefaultHasherSeed) -> Self {
        Self {
            seed: rkyv_0_8::rend::u64_le::from_native(seed.seed),
        }
    }
}
