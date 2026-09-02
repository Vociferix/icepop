//! A portable hasher and its seed, built on rapidhash.

use rapidhash::quality::{RapidHasher, SeedableState};

use core::hash::{BuildHasher, Hasher};

use crate::hash::PortableHash;
use crate::hasher::PortableHasher;

/// Seed for [`DefaultHasher`], and the [`BuildHasher`] that builds it.
///
/// Equal seeds build hashers that produce identical hashes on every platform, so a seed can be
/// stored or sent to another machine to reproduce hashes there.
///
/// # Example
///
/// ```
/// use portable::{DefaultHasherSeed, PortableBuildHasher};
///
/// let seed = DefaultHasherSeed::new();
/// let same_seed = DefaultHasherSeed::with_seed(seed.seed());
///
/// assert_eq!(seed.portable_hash_one("hello"), same_seed.portable_hash_one("hello"));
/// ```
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
#[cfg_attr(
    feature = "rkyv-0_8",
    rkyv(attr(
        doc = "The archived form of [`DefaultHasherSeed`], usable as a [`BuildHasher`] as is.",
        doc = "",
        doc = "Builds the same [`DefaultHasher`] as the seed it was archived from.",
        doc = "",
        doc = "# Example",
        doc = "",
        doc = "```",
        doc = "use portable::{ArchivedDefaultHasherSeed, DefaultHasherSeed, PortableBuildHasher};",
        doc = "",
        doc = "let seed = DefaultHasherSeed::with_seed(42);",
        doc = "let archived = ArchivedDefaultHasherSeed::with_seed(42);",
        doc = "",
        doc = "assert_eq!(seed.portable_hash_one(\"hello\"), archived.portable_hash_one(\"hello\"));",
        doc = "```"
    ))
)]
pub struct DefaultHasherSeed {
    #[cfg_attr(feature = "rkyv-0_8", rkyv(omit_bounds))]
    seed: u64,
}

/// A [`PortableHasher`] based on [rapidhash](https://docs.rs/rapidhash).
///
/// Built from a [`DefaultHasherSeed`]; hashers built from equal seeds produce identical hashes
/// on every platform.
///
/// # Example
///
/// ```
/// use core::hash::{BuildHasher, Hasher};
/// use portable::{DefaultHasherSeed, PortableHash};
///
/// let seed = DefaultHasherSeed::with_seed(42);
///
/// let mut hasher = seed.build_hasher();
/// "hello".portable_hash(&mut hasher);
///
/// let mut other = seed.build_hasher();
/// "hello".portable_hash(&mut other);
///
/// assert_eq!(hasher.finish(), other.finish());
/// ```
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
    /// Creates a randomly generated seed.
    ///
    /// With the `getrandom` feature the randomness comes from the operating system, drawn once
    /// per process and advanced for each later seed; otherwise it comes from a value generated
    /// at compile time, which is the same for every run of the same binary.
    ///
    /// # Panics
    ///
    /// Panics if the operating system's entropy source is unavailable (`getrandom` feature
    /// only).
    ///
    /// # Example
    ///
    /// ```
    /// use portable::{DefaultHasherSeed, PortableBuildHasher};
    ///
    /// let seed = DefaultHasherSeed::new();
    ///
    /// assert_eq!(seed.portable_hash_one("hello"), seed.portable_hash_one("hello"));
    /// ```
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

    /// Creates a seed from an explicit value.
    ///
    /// # Example
    ///
    /// ```
    /// use portable::{DefaultHasherSeed, PortableBuildHasher};
    ///
    /// let seed = DefaultHasherSeed::with_seed(42);
    /// let same_seed = DefaultHasherSeed::with_seed(42);
    ///
    /// assert_eq!(seed.portable_hash_one("hello"), same_seed.portable_hash_one("hello"));
    /// ```
    #[inline]
    pub const fn with_seed(seed: u64) -> Self {
        Self { seed }
    }

    /// Returns the seed value.
    ///
    /// # Example
    ///
    /// ```
    /// use portable::DefaultHasherSeed;
    ///
    /// assert_eq!(DefaultHasherSeed::with_seed(42).seed(), 42);
    /// ```
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
    /// Creates a randomly generated seed, as [`DefaultHasherSeed::new`] does.
    ///
    /// # Panics
    ///
    /// Panics if the operating system's entropy source is unavailable (`getrandom` feature
    /// only).
    ///
    /// # Example
    ///
    /// ```
    /// use portable::{ArchivedDefaultHasherSeed, PortableBuildHasher};
    ///
    /// let seed = ArchivedDefaultHasherSeed::new();
    ///
    /// assert_eq!(seed.portable_hash_one("hello"), seed.portable_hash_one("hello"));
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self {
            seed: rkyv_0_8::rend::u64_le::from_native(DefaultHasherSeed::new().seed),
        }
    }

    /// Creates a seed from an explicit value.
    ///
    /// # Example
    ///
    /// ```
    /// use portable::{ArchivedDefaultHasherSeed, DefaultHasherSeed, PortableBuildHasher};
    ///
    /// let archived = ArchivedDefaultHasherSeed::with_seed(42);
    /// let seed = DefaultHasherSeed::with_seed(42);
    ///
    /// assert_eq!(archived.portable_hash_one("hello"), seed.portable_hash_one("hello"));
    /// ```
    #[inline]
    pub const fn with_seed(seed: u64) -> Self {
        Self {
            seed: rkyv_0_8::rend::u64_le::from_native(seed),
        }
    }

    /// Returns the seed value.
    ///
    /// # Example
    ///
    /// ```
    /// use portable::ArchivedDefaultHasherSeed;
    ///
    /// assert_eq!(ArchivedDefaultHasherSeed::with_seed(42).seed(), 42);
    /// ```
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

#[cfg(all(feature = "rkyv-0_8", feature = "serde"))]
impl serde::Serialize for ArchivedDefaultHasherSeed {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut ser = serializer.serialize_struct("DefaultHasherSeed", 1)?;
        ser.serialize_field("seed", &self.seed.to_native())?;
        ser.end()
    }
}

#[cfg(test)]
mod tests {
    use super::{DefaultHasher, DefaultHasherSeed};

    use crate::hash::PortableHash;
    use crate::hasher::PortableBuildHasher;

    use core::hash::{BuildHasher, Hasher};

    fn finish(state: impl FnOnce(&mut DefaultHasher)) -> u64 {
        let mut hasher = DefaultHasherSeed::with_seed(42).build_hasher();
        state(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn a_seed_round_trips_through_its_value() {
        assert_eq!(DefaultHasherSeed::with_seed(42).seed(), 42);
    }

    #[test]
    fn equal_seeds_hash_equally_and_different_seeds_do_not() {
        let seed = DefaultHasherSeed::new();
        let same = DefaultHasherSeed::with_seed(seed.seed());

        assert_eq!(
            seed.portable_hash_one("hello"),
            same.portable_hash_one("hello")
        );
        assert_ne!(
            seed.portable_hash_one("hello"),
            DefaultHasherSeed::with_seed(seed.seed().wrapping_add(1)).portable_hash_one("hello"),
        );
        assert_ne!(
            seed.portable_hash_one("hello"),
            seed.portable_hash_one("world")
        );
    }

    #[test]
    fn each_generated_seed_differs_from_the_last() {
        let first = DefaultHasherSeed::new();
        let second = DefaultHasherSeed::new();

        assert_ne!(first.seed(), second.seed());
    }

    #[test]
    fn integer_writes_are_normalized_to_their_portable_encoding() {
        // A `Hasher` used directly must not reintroduce the platform's integer encoding.
        assert_eq!(
            finish(|state| state.write_u16(1)),
            finish(|state| state.write_u64(1))
        );
        assert_eq!(
            finish(|state| state.write_u32(1)),
            finish(|state| state.write_usize(1))
        );
        assert_eq!(
            finish(|state| state.write_i16(-1)),
            finish(|state| state.write_i64(-1))
        );
        assert_eq!(
            finish(|state| state.write_i32(-1)),
            finish(|state| state.write_isize(-1))
        );

        assert_eq!(
            finish(|state| state.write_u32(1)),
            finish(|state| 1u32.portable_hash(state))
        );
        assert_ne!(
            finish(|state| state.write_u32(1)),
            finish(|state| state.write_u32(2))
        );
    }

    #[cfg(feature = "rkyv-0_8")]
    #[test]
    fn an_archived_seed_builds_the_same_hasher_as_the_seed_it_came_from() {
        use super::ArchivedDefaultHasherSeed;

        let seed = DefaultHasherSeed::with_seed(42);
        let archived = ArchivedDefaultHasherSeed::with_seed(42);

        assert_eq!(archived.seed(), 42);
        assert_eq!(
            seed.portable_hash_one("hello"),
            archived.portable_hash_one("hello")
        );

        assert_eq!(DefaultHasherSeed::from(archived).seed(), 42);
        assert_eq!(ArchivedDefaultHasherSeed::from(seed).seed(), 42);
    }

    #[cfg(feature = "serde")]
    mod serde {
        use super::DefaultHasherSeed;

        use ::serde::Deserialize;
        use ::serde::de::value::{Error, MapDeserializer, SeqDeserializer};

        fn from_map<'a>(
            fields: impl IntoIterator<Item = (&'a str, u64)>,
        ) -> Result<DefaultHasherSeed, Error> {
            DefaultHasherSeed::deserialize(MapDeserializer::<_, Error>::new(fields.into_iter()))
        }

        #[test]
        fn a_seed_deserializes_from_a_map() {
            assert_eq!(from_map([("seed", 42u64)]).unwrap().seed(), 42);
        }

        #[test]
        fn the_field_name_is_matched_ignoring_case() {
            assert_eq!(from_map([("SEED", 42u64)]).unwrap().seed(), 42);
        }

        #[test]
        fn a_seed_deserializes_from_a_sequence() {
            let values = SeqDeserializer::<_, Error>::new([42u64].into_iter());

            assert_eq!(DefaultHasherSeed::deserialize(values).unwrap().seed(), 42);
        }

        #[test]
        fn an_unknown_field_is_rejected() {
            assert!(from_map([("other", 42u64)]).is_err());
        }

        #[test]
        fn a_missing_field_is_rejected() {
            assert!(from_map([]).is_err());
        }

        #[test]
        fn a_duplicated_field_is_rejected() {
            assert!(from_map([("seed", 42u64), ("seed", 43)]).is_err());
        }
    }
}
