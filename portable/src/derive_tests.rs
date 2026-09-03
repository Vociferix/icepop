//! Tests for the derive macros.
//!
//! The crate refers to itself as `portable` under `cfg(test)`, so every type here is written
//! exactly as a dependent writes it, with no `crate` attribute unless the test is about one.

use core::hash::Hasher;
use std::collections::hash_map::DefaultHasher;

use crate::eq::PortableReprEq;
use crate::ord::PortableReprOrd;
use crate::repr::VisitPortableRepr;
use crate::{PortableEq, PortableHash, PortableOrd};

fn hash_of(value: &impl PortableHash) -> u64 {
    let mut state = DefaultHasher::new();
    value.portable_hash(&mut state);
    state.finish()
}

mod representations {
    use super::*;

    /// The default: the type is its own representation.
    #[derive(VisitPortableRepr, PortableReprEq)]
    struct Own {
        a: u32,
    }

    /// The default spelled out.
    #[derive(VisitPortableRepr, PortableReprEq)]
    #[portable(repr = Self)]
    struct OwnExplicit {
        a: u32,
    }

    /// A representation reached by a map, rather than generated.
    #[derive(VisitPortableRepr)]
    #[portable(repr = u32, map = |id: &Delegating| id.0)]
    struct Delegating(u32);

    /// The same, written with the parenthesised form.
    #[derive(VisitPortableRepr)]
    #[portable(repr(u32), map(|id: &DelegatingParens| id.0))]
    struct DelegatingParens(u32);

    #[derive(VisitPortableRepr, PortableReprEq)]
    #[portable(repr)]
    struct Generated {
        a: u32,
    }

    #[derive(VisitPortableRepr, PortableReprEq)]
    #[portable(repr(ChosenName))]
    struct NamedParens(u32);

    #[derive(VisitPortableRepr, PortableReprEq)]
    #[portable(repr = OtherChosenName)]
    struct NamedEquals(u32);

    /// The crate path is configurable, and `crate` itself is a valid path.
    #[derive(VisitPortableRepr, PortableReprEq)]
    #[portable(crate = crate, repr)]
    struct CratePath(u32);

    #[derive(VisitPortableRepr, PortableReprEq)]
    #[portable(crate(crate), repr)]
    struct CratePathParens(u32);

    #[test]
    fn a_type_represents_itself_by_default() {
        let visited = Own { a: 7 }.visit_portable_repr(|repr| repr.a);

        assert_eq!(visited, 7);
        assert!(Own { a: 7 }.portable_eq(&Own { a: 7 }));
        assert!(OwnExplicit { a: 7 }.portable_eq(&OwnExplicit { a: 7 }));
    }

    #[test]
    fn a_mapped_type_gains_every_comparison_of_what_it_maps_to() {
        assert!(Delegating(7).portable_eq(&7u32));
        assert!(Delegating(7).portable_cmp(&8u32).is_lt());
        assert!(DelegatingParens(7).portable_eq(&7u32));

        // `usize` shares a representation with `u32`, so it reaches through as well.
        assert!(Delegating(7).portable_eq(&7usize));
    }

    #[test]
    fn a_generated_representation_borrows_each_field() {
        let doubled = Generated { a: 21 }.visit_portable_repr(|repr| *repr.a * 2);

        assert_eq!(doubled, 42);
    }

    #[test]
    fn a_generated_representation_takes_the_name_it_is_given() {
        // Naming the types is the assertion: they only exist if the derive named them so.
        let _: Option<GeneratedRepr<u32>> = None;
        let _: Option<ChosenName<u32>> = None;
        let _: Option<OtherChosenName<u32>> = None;

        assert!(NamedParens(1).portable_eq(&NamedParens(1)));
        assert!(NamedEquals(1).portable_eq(&NamedEquals(1)));
    }

    #[test]
    fn the_crate_path_may_be_given_explicitly() {
        assert!(CratePath(1).portable_eq(&CratePath(1)));
        assert!(CratePathParens(1).portable_eq(&CratePathParens(1)));
    }

    #[test]
    fn a_representation_field_compares_and_hashes_as_the_field_itself() {
        Generated { a: 5 }.visit_portable_repr(|repr| {
            assert!(repr.a.portable_eq(&5u32));
            assert!(repr.a.portable_cmp(&9u32).is_lt());
            assert_eq!(hash_of(&repr.a), hash_of(&5u32));
        });
    }
}

mod shapes {
    use super::*;

    #[derive(VisitPortableRepr, PortableReprEq, PortableReprOrd, PortableHash)]
    #[portable(repr)]
    struct Named {
        first: u32,
        second: u32,
    }

    #[derive(VisitPortableRepr, PortableReprEq, PortableReprOrd, PortableHash)]
    #[portable(repr)]
    struct Tuple(u32, u32);

    #[derive(VisitPortableRepr, PortableReprEq, PortableReprOrd, PortableHash)]
    #[portable(repr)]
    struct Unit;

    #[derive(VisitPortableRepr, PortableReprEq, PortableReprOrd, PortableHash)]
    #[portable(repr)]
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
    enum Enum {
        Nothing,
        One(u32),
        Two(u32, u32),
        Struct { w: u32, h: u32 },
    }

    /// An empty enum has no values, but the derives must still expand.
    #[derive(VisitPortableRepr, PortableReprEq, PortableReprOrd, PortableHash)]
    #[portable(repr)]
    enum Empty {}

    /// A single-variant enum must not warn about the unreachable mismatch arm.
    #[derive(VisitPortableRepr, PortableReprEq, PortableReprOrd, PortableHash)]
    #[portable(repr)]
    enum Single {
        Only(u32),
    }

    #[test]
    fn structs_compare_field_by_field() {
        let base = Named {
            first: 1,
            second: 2,
        };

        assert!(base.portable_eq(&Named {
            first: 1,
            second: 2
        }));
        assert!(!base.portable_eq(&Named {
            first: 1,
            second: 3
        }));
        assert!(Tuple(1, 2).portable_eq(&Tuple(1, 2)));
        assert!(!Tuple(1, 2).portable_eq(&Tuple(2, 1)));
    }

    #[test]
    fn earlier_fields_dominate_the_ordering() {
        let low = Named {
            first: 1,
            second: 9,
        };
        let high = Named {
            first: 2,
            second: 0,
        };

        assert!(low.portable_cmp(&high).is_lt());
        assert!(high.portable_cmp(&low).is_gt());
        assert!(low.portable_cmp(&low).is_eq());
        assert!(Tuple(1, 9).portable_cmp(&Tuple(2, 0)).is_lt());
    }

    #[test]
    fn a_unit_struct_is_equal_to_itself_and_never_ordered() {
        assert!(Unit.portable_eq(&Unit));
        assert!(Unit.portable_cmp(&Unit).is_eq());
        assert_eq!(hash_of(&Unit), hash_of(&Unit));
    }

    #[test]
    fn enum_variants_are_equal_only_to_themselves() {
        assert!(Enum::Nothing.portable_eq(&Enum::Nothing));
        assert!(Enum::One(1).portable_eq(&Enum::One(1)));
        assert!(!Enum::One(1).portable_eq(&Enum::One(2)));
        assert!(!Enum::One(1).portable_eq(&Enum::Nothing));
        assert!(Enum::Struct { w: 1, h: 2 }.portable_eq(&Enum::Struct { w: 1, h: 2 }));
        assert!(!Enum::Struct { w: 1, h: 2 }.portable_eq(&Enum::Two(1, 2)));
    }

    #[test]
    fn enum_ordering_matches_what_ord_would_derive() {
        let all = [
            Enum::Nothing,
            Enum::One(0),
            Enum::One(7),
            Enum::Two(0, 0),
            Enum::Two(0, 1),
            Enum::Two(1, 0),
            Enum::Struct { w: 0, h: 0 },
            Enum::Struct { w: 0, h: 1 },
            Enum::Struct { w: 1, h: 0 },
        ];

        for left in &all {
            for right in &all {
                assert_eq!(
                    left.portable_cmp(right),
                    left.cmp(right),
                    "{left:?} vs {right:?}"
                );
                assert_eq!(
                    left.portable_eq(right),
                    left == right,
                    "{left:?} vs {right:?}"
                );
            }
        }
    }

    #[test]
    fn a_single_variant_enum_still_compares() {
        assert!(Single::Only(1).portable_eq(&Single::Only(1)));
        assert!(Single::Only(1).portable_cmp(&Single::Only(2)).is_lt());
    }

    #[test]
    fn an_empty_enum_expands() {
        fn assert_impls<T>()
        where
            T: VisitPortableRepr + PortableHash,
            T::Repr: PortableReprEq + PortableReprOrd,
        {
        }

        assert_impls::<Empty>();
    }
}

mod hashing {
    use super::*;

    #[derive(PortableHash)]
    struct Fields {
        a: u32,
        b: u64,
    }

    #[derive(PortableHash)]
    enum Tagged {
        First(u8),
        Second(u8),
    }

    #[test]
    fn equal_values_hash_equally() {
        assert_eq!(
            hash_of(&Fields { a: 1, b: 2 }),
            hash_of(&Fields { a: 1, b: 2 })
        );
        assert_ne!(
            hash_of(&Fields { a: 1, b: 2 }),
            hash_of(&Fields { a: 1, b: 3 })
        );
    }

    #[test]
    fn the_variant_is_hashed_as_well_as_the_payload() {
        // Identical payloads in different variants must not collide.
        assert_ne!(hash_of(&Tagged::First(1)), hash_of(&Tagged::Second(1)));
        assert_eq!(hash_of(&Tagged::First(1)), hash_of(&Tagged::First(1)));
        assert_ne!(hash_of(&Tagged::First(1)), hash_of(&Tagged::First(2)));
    }

    #[test]
    fn a_derived_option_shaped_enum_hashes_like_option() {
        #[derive(PortableHash)]
        enum MyOption {
            None,
            Some(u32),
        }

        // Variants are tagged by declaration index, which lines up with the hand-written
        // `Option` impl's boolean tag.
        assert_eq!(hash_of(&MyOption::None), hash_of(&None::<u32>));
        assert_eq!(hash_of(&MyOption::Some(7)), hash_of(&Some(7u32)));
    }
}

mod generics {
    use super::*;

    #[derive(VisitPortableRepr, PortableReprEq, PortableReprOrd, PortableHash)]
    #[portable(repr)]
    struct Generated<T> {
        value: T,
        count: u32,
    }

    /// Without a generated representation the derives bound each parameter themselves.
    #[derive(VisitPortableRepr, PortableReprEq, PortableReprOrd, PortableHash)]
    struct Own<T> {
        value: T,
    }

    #[test]
    fn generic_types_compare_and_hash() {
        let left = Generated {
            value: 1u32,
            count: 1,
        };
        let right = Generated {
            value: 2u32,
            count: 0,
        };

        assert!(left.portable_eq(&left));
        assert!(!left.portable_eq(&right));
        assert!(left.portable_cmp(&right).is_lt());
        assert_eq!(hash_of(&left), hash_of(&left));
    }

    #[test]
    fn a_generic_type_that_represents_itself_compares() {
        assert!(Own { value: 1u32 }.portable_eq(&Own { value: 1u32 }));
        assert!(
            Own { value: 1u32 }
                .portable_cmp(&Own { value: 2u32 })
                .is_lt()
        );
    }
}

mod bounds {
    use super::*;

    trait Marker {
        type Assoc;
    }

    impl Marker for u32 {
        type Assoc = u8;
    }

    /// Custom bounds are additive: `T: Marker` is declared by the type and must survive, or
    /// `T::Assoc` would not resolve.
    #[derive(VisitPortableRepr, PortableReprEq, PortableReprOrd, PortableHash)]
    #[portable(
        repr = Self,
        bounds(T::Assoc: PortableEq + PortableOrd + PortableHash)
    )]
    struct Declared<T: Marker> {
        value: T::Assoc,
    }

    /// A `where` clause the type already carries must survive too.
    #[derive(VisitPortableRepr, PortableReprEq, PortableHash)]
    #[portable(repr = Self, bounds(T::Assoc: PortableEq + PortableHash))]
    struct WhereClause<T>
    where
        T: Marker,
    {
        value: T::Assoc,
    }

    /// A type that can be hashed but not compared, to show which bound actually applied.
    struct HashOnly(u32);

    impl PortableHash for HashOnly {
        fn portable_hash<H: core::hash::Hasher>(&self, state: &mut H) {
            self.0.portable_hash(state);
        }
    }

    /// The per-derive attribute takes precedence over the shared one: hashing needs only
    /// `PortableHash`, so `PerDerive<HashOnly>` hashes even though it cannot be compared.
    #[derive(VisitPortableRepr, PortableReprEq, PortableReprOrd, PortableHash)]
    #[portable(
        repr = Self,
        bounds(T: PortableEq + PortableOrd + PortableHash),
        hash_bounds(T: PortableHash)
    )]
    struct PerDerive<T> {
        value: T,
    }

    /// Lifetime bounds declared by the type must survive as well.
    #[derive(VisitPortableRepr, PortableReprEq, PortableHash)]
    #[portable(repr = Self, bounds())]
    struct Lifetimes<'a, 'b: 'a> {
        first: &'a str,
        second: &'b str,
    }

    #[test]
    fn declared_parameter_bounds_survive_custom_bounds() {
        let left = Declared::<u32> { value: 1 };
        let right = Declared::<u32> { value: 2 };

        assert!(left.portable_eq(&left));
        assert!(left.portable_cmp(&right).is_lt());
        assert_eq!(hash_of(&left), hash_of(&left));
    }

    #[test]
    fn a_declared_where_clause_survives_custom_bounds() {
        let value = WhereClause::<u32> { value: 1 };

        assert!(value.portable_eq(&value));
    }

    #[test]
    fn per_derive_bounds_override_the_shared_ones() {
        let left = PerDerive { value: 1u32 };
        let right = PerDerive { value: 2u32 };

        assert!(left.portable_eq(&left));
        assert!(left.portable_cmp(&right).is_lt());

        // `HashOnly` satisfies `hash_bounds` but not the shared `bounds`, so this only
        // compiles because the narrower per-derive attribute won.
        assert_eq!(
            hash_of(&PerDerive { value: HashOnly(1) }),
            hash_of(&PerDerive { value: HashOnly(1) })
        );
    }

    #[test]
    fn declared_lifetime_bounds_survive_custom_bounds() {
        let value = Lifetimes {
            first: "a",
            second: "b",
        };

        assert!(value.portable_eq(&value));
    }
}

#[cfg(all(feature = "rkyv-0_8", feature = "alloc"))]
mod rkyv {
    use super::*;

    use core::cmp::Ordering;

    use rkyv_0_8::{Archive, Serialize};

    /// Serialises `value` and binds `$name` to a view of its archived form.
    ///
    /// The buffer is bound in the caller's scope so that it outlives the view into it.
    macro_rules! archived {
        ($name:ident, $value:expr, $archived:ty) => {
            let bytes = rkyv_0_8::to_bytes::<rkyv_0_8::rancor::Error>(&$value).unwrap();
            // SAFETY: the bytes were just produced by `to_bytes` from a value of the matching
            // type, so they hold a valid archived value at their root.
            let $name = unsafe { rkyv_0_8::access_unchecked::<$archived>(&bytes) };
        };
    }

    #[derive(
        Archive, Serialize, VisitPortableRepr, PortableReprEq, PortableReprOrd, PortableHash,
    )]
    #[rkyv(crate = ::rkyv_0_8)]
    #[portable(rkyv, rkyv_crate = ::rkyv_0_8)]
    struct Point {
        x: u32,
        y: u64,
    }

    #[derive(
        Archive, Serialize, VisitPortableRepr, PortableReprEq, PortableReprOrd, PortableHash,
    )]
    #[rkyv(crate = ::rkyv_0_8)]
    #[portable(rkyv, rkyv_crate(::rkyv_0_8))]
    enum Shape {
        Nothing,
        One(u32),
        Struct { w: u32, h: u32 },
    }

    /// The archived name comes from rkyv's own attribute when it is given one.
    #[derive(Archive, Serialize, VisitPortableRepr, PortableReprEq)]
    #[rkyv(crate = ::rkyv_0_8, archived = TheArchive)]
    #[portable(rkyv, rkyv_crate = ::rkyv_0_8)]
    struct Renamed {
        v: u32,
    }

    /// ...and may also be given directly, alongside a chosen representation name.
    #[derive(Archive, Serialize, VisitPortableRepr, PortableReprEq)]
    #[rkyv(crate = ::rkyv_0_8, archived = BothNamed)]
    #[portable(repr(BothView), rkyv = BothNamed, rkyv_crate = ::rkyv_0_8)]
    struct Both {
        v: u32,
    }

    #[derive(
        Archive, Serialize, VisitPortableRepr, PortableReprEq, PortableReprOrd, PortableHash,
    )]
    #[rkyv(crate = ::rkyv_0_8)]
    #[portable(rkyv, rkyv_crate = ::rkyv_0_8)]
    struct Generic<T> {
        value: T,
        count: u32,
    }

    #[test]
    fn a_value_and_its_archived_form_compare_in_both_directions() {
        let point = Point { x: 1, y: 2 };
        archived!(archived, point, ArchivedPoint);

        assert!(point.portable_eq(archived));
        assert!(archived.portable_eq(&point));
        assert!(point.portable_cmp(archived).is_eq());
        assert!(archived.portable_cmp(&point).is_eq());
    }

    #[test]
    fn ordering_against_an_archived_value_is_symmetric() {
        archived!(archived, Point { x: 1, y: 2 }, ArchivedPoint);

        let greater = Point { x: 1, y: 3 };
        assert!(greater.portable_cmp(archived).is_gt());
        assert!(archived.portable_cmp(&greater).is_lt());

        let lesser = Point { x: 0, y: 9 };
        assert!(lesser.portable_cmp(archived).is_lt());
        assert!(archived.portable_cmp(&lesser).is_gt());
    }

    #[test]
    fn a_value_and_its_archived_form_hash_equally() {
        let point = Point { x: 1, y: 2 };
        archived!(archived, point, ArchivedPoint);

        assert_eq!(hash_of(&point), hash_of(archived));
    }

    #[test]
    fn every_enum_variant_survives_the_round_trip() {
        for shape in [Shape::Nothing, Shape::One(7), Shape::Struct { w: 1, h: 2 }] {
            archived!(archived, shape, ArchivedShape);

            assert!(shape.portable_eq(archived));
            assert!(archived.portable_eq(&shape));
            assert!(shape.portable_cmp(archived).is_eq());
            assert_eq!(hash_of(&shape), hash_of(archived));
        }
    }

    #[test]
    fn archived_variants_order_by_declaration_like_native_ones() {
        archived!(archived, Shape::One(5), ArchivedShape);

        assert!(Shape::Nothing.portable_cmp(archived).is_lt());
        assert!(Shape::One(4).portable_cmp(archived).is_lt());
        assert!(Shape::Struct { w: 0, h: 0 }.portable_cmp(archived).is_gt());
        assert!(archived.portable_cmp(&Shape::Nothing).is_gt());
    }

    #[test]
    fn the_archived_type_may_be_named_by_either_crate() {
        let renamed = Renamed { v: 9 };
        archived!(archived, renamed, TheArchive);
        assert!(renamed.portable_eq(archived));

        let both = Both { v: 9 };
        archived!(archived_both, both, BothNamed);
        let archived = archived_both;
        assert!(both.portable_eq(archived));
        let _: Option<BothView<u32>> = None;
    }

    #[test]
    fn generic_types_interoperate_with_their_archived_form() {
        let value = Generic {
            value: 1u32,
            count: 2,
        };
        archived!(archived, value, ArchivedGeneric<u32>);

        assert!(value.portable_eq(archived));
        assert!(archived.portable_eq(&value));
        assert_eq!(hash_of(&value), hash_of(archived));
    }

    #[test]
    fn ordering_is_transitive_across_native_and_archived_values() {
        archived!(middle, Point { x: 1, y: 5 }, ArchivedPoint);

        let low = Point { x: 1, y: 4 };
        let high = Point { x: 1, y: 6 };

        assert_eq!(low.portable_cmp(middle), Ordering::Less);
        assert_eq!(middle.portable_cmp(&high), Ordering::Less);
        assert_eq!(low.portable_cmp(&high), Ordering::Less);
    }
}
