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

extern crate alloc;

pub use portability::{NonPortable, Portable};

#[doc(inline)]
pub use self::{
    map::{Map, PortableMap},
    ordered_map::{OrderedMap, PortableOrderedMap},
    ordered_set::{OrderedSet, PortableOrderedSet},
    set::{PortableSet, Set},
};

#[cfg(feature = "rkyv")]
pub mod rkyv {
    #[doc(inline)]
    pub use super::{
        map::rkyv::ArchivedMap, ordered_map::rkyv::ArchivedOrderedMap,
        ordered_set::rkyv::ArchivedOrderedSet, set::rkyv::ArchivedSet,
    };
}
