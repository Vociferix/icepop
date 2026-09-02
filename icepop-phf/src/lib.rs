//! Minimal perfect hash maps and sets: built once, then read-only.
//!
//! A minimal perfect hash function maps a known set of keys onto `0..len` with no collisions
//! and no gaps, so a lookup examines exactly one entry and the table stores no empty slots.
//! The cost is that the whole key set must be known up front and the result cannot be modified
//! afterwards. Entries are accumulated in a `Builder`, an ordinary mutable hash table, and
//! frozen by calling `build`.
//!
//! # Collections
//!
//! |                 | map            | set            |
//! |-----------------|----------------|----------------|
//! | arbitrary order | [`Map`]        | [`Set`]        |
//! | insertion order | [`OrderedMap`] | [`OrderedSet`] |
//!
//! The `Ordered` collections iterate, slice and index in the order entries were inserted into
//! the builder, spending four bytes per entry and one extra indirection per lookup to do so.
//! The others permute entries so that a key's hash slot is its index, and iterate in an
//! arbitrary order.
//!
//! # Portability
//!
//! The final type parameter selects the lookup interface and is either [`NonPortable`] or
//! [`Portable`].
//!
//! [`NonPortable`], the default, uses [`Hash`](core::hash::Hash),
//! [`BuildHasher`](core::hash::BuildHasher), [`Eq`] and [`Equivalent`], the same interface the
//! standard library's `HashMap` offers. Nothing constrains the hasher, so it may be the
//! fastest one available, but the collection means nothing outside the process that built it
//! and supports neither `serde` nor `rkyv`.
//!
//! [`Portable`] uses [`PortableHash`], [`PortableEq`] and [`PortableBuildHasher`] from the
//! re-exported [`portable`] crate, which hash and compare identically on every platform. The
//! [`PortableMap`], [`PortableSet`], [`PortableOrderedMap`] and [`PortableOrderedSet`] aliases
//! name these forms.
//!
//! # Hashing
//!
//! Building a minimal perfect hash function requires that distinct keys produce distinct
//! 64-bit hashes. Distinct keys that collide are not resolved by probing, as they would be in
//! an ordinary hash table; the builder rehashes the whole key set under a different parameter
//! instead, and gives up with a panic once every parameter has failed. Any reasonable hasher
//! satisfies this, but a truncating or trivially weak one does not.
//!
//! The hasher is stored in the collection and reused for every lookup, so it must return the
//! same hash for the same key for as long as the collection is read.
//!
//! # Transferring a built collection
//!
//! A [`Portable`] collection can be built by one process and read by another, on a machine
//! with different endianness, pointer width or operating system. This holds because
//! [`PortableHash`] and [`PortableBuildHasher`] fix the bytes fed to the hasher and the hash
//! that comes back, and because the hasher's seed travels with the collection.
//!
//! Neither serialization backend is enabled by default. With the `serde` feature, the
//! collection serializes and deserializes as an ordinary struct. With the `rkyv` feature it also
//! archives, and the archived form answers the full read interface directly out of the
//! serialized bytes, with no deserialization step and no copies.
#![cfg_attr(feature = "rkyv", doc = "See [`rkyv::ArchivedMap`] and its siblings.")]
//!
//! # Example
//!
//! ```
//! use icepop_phf::Map;
//!
//! let mut builder = Map::<&str, u32>::builder();
//! builder.insert("red", 0xff0000);
//! builder.insert("green", 0x00ff00);
//! builder.insert("blue", 0x0000ff);
//!
//! let map = builder.build();
//!
//! assert_eq!(map.get("green"), Some(&0x00ff00));
//! assert_eq!(map.get("purple"), None);
//! assert_eq!(map.len(), 3);
//! ```
//!
//! # Features
//!
//! - `getrandom` *(default)*: Seeds [`DefaultHasherSeed::new`] from the operating system's
//!   entropy source instead of a compile-time random value.
//! - `rkyv`: Archives the [`Portable`] collections with
//!   [rkyv 0.8](https://docs.rs/rkyv/0.8/rkyv/), and provides the `rkyv` module of archived
//!   forms that serve lookups directly from a serialized buffer.
//! - `serde`: Implements [serde](https://docs.rs/serde/1/serde/) `Serialize` and
//!   `Deserialize` for the [`Portable`] collections, and `Serialize` for their archived forms.
//!
//! The crate is `no_std`, but always requires `alloc`.
//!
//! [`Equivalent`]: https://docs.rs/equivalent/1/equivalent/trait.Equivalent.html
//! [`PortableBuildHasher`]: portable::PortableBuildHasher
//! [`PortableEq`]: portable::PortableEq
//! [`PortableHash`]: portable::PortableHash

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod map;
pub mod ordered_map;
pub mod ordered_set;
pub mod set;

mod map_iters;
mod portability;
mod set_iters;
mod table;

#[doc(inline)]
pub use portable::{DefaultHasher, DefaultHasherSeed};

/// The traits that define what [`Portable`] means, re-exported for convenience.
///
/// Key types and hashers used with a [`Portable`] collection must implement these, so this
/// re-export saves depending on the crate directly.
pub use portable;

extern crate alloc;

#[cfg(test)]
extern crate std;

pub use portability::{NonPortable, Portable};

#[doc(inline)]
pub use self::{
    map::{Map, PortableMap},
    ordered_map::{OrderedMap, PortableOrderedMap},
    ordered_set::{OrderedSet, PortableOrderedSet},
    set::{PortableSet, Set},
};

/// The archived forms of the [`Portable`] collections.
///
/// Each answers the full read interface of the collection it was archived from, reading
/// directly out of the serialized bytes. Obtain one with [`rkyv::access`](::rkyv::access)
/// over a buffer produced by [`rkyv::to_bytes`](::rkyv::to_bytes).
///
/// # Example
///
/// ```
/// use icepop_phf::{PortableSet, rkyv::ArchivedSet};
/// use rkyv::rancor::Error;
///
/// let set: PortableSet<String> = ["kiwi".to_string(), "fig".to_string()].into_iter().collect();
/// let bytes = rkyv::to_bytes::<Error>(&set)?;
/// let archived = rkyv::access::<ArchivedSet<String>, Error>(&bytes)?;
///
/// assert!(archived.contains("fig"));
/// assert!(!archived.contains("plum"));
/// # Ok::<(), Error>(())
/// ```
#[cfg(feature = "rkyv")]
pub mod rkyv {
    #[doc(inline)]
    pub use super::{
        map::rkyv::ArchivedMap, ordered_map::rkyv::ArchivedOrderedMap,
        ordered_set::rkyv::ArchivedOrderedSet, set::rkyv::ArchivedSet,
    };
}
