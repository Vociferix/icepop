use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Expr, Fields, GenericParam, Generics, Ident, Index, Path, Result,
    Token, Type, TypeParamBound, WherePredicate,
    ext::IdentExt,
    parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
};

use core::borrow::Borrow;

enum Attr {
    Crate(Path),
    Bounds(Punctuated<WherePredicate, Token![,]>),
    HashBounds(Punctuated<WherePredicate, Token![,]>),
    ReprEqBounds(Punctuated<WherePredicate, Token![,]>),
    ReprOrdBounds(Punctuated<WherePredicate, Token![,]>),
    VisitReprBounds(Punctuated<WherePredicate, Token![,]>),
    Repr(Type),
    Map(Expr),
}

struct Attrs {
    crate_path: Path,
    bounds: Option<Punctuated<WherePredicate, Token![,]>>,
    hash_bounds: Option<Punctuated<WherePredicate, Token![,]>>,
    repr_eq_bounds: Option<Punctuated<WherePredicate, Token![,]>>,
    repr_ord_bounds: Option<Punctuated<WherePredicate, Token![,]>>,
    visit_repr_bounds: Option<Punctuated<WherePredicate, Token![,]>>,
    repr: Option<Type>,
    map: Option<Expr>,
}

impl Parse for Attr {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let ident = Ident::parse_any(input)?;
        let val = if ident == "crate" {
            if input.peek(Token![=]) {
                input.parse::<Token![=]>()?;
                Self::Crate(input.parse()?)
            } else {
                let content;
                let _paren = parenthesized!(content in input);
                let attr = Self::Crate(content.parse()?);
                if !content.is_empty() {
                    return Err(syn::Error::new(
                        content.span(),
                        "unexpected tokens in attribute",
                    ));
                }
                attr
            }
        } else if ident == "bounds" {
            let content;
            let _paren = parenthesized!(content in input);
            let attr = Self::Bounds(Punctuated::parse_terminated(&content)?);
            if !content.is_empty() {
                return Err(syn::Error::new(
                    content.span(),
                    "unexpected tokens in attribute",
                ));
            }
            attr
        } else if ident == "hash_bounds" {
            let content;
            let _paren = parenthesized!(content in input);
            let attr = Self::HashBounds(Punctuated::parse_terminated(&content)?);
            if !content.is_empty() {
                return Err(syn::Error::new(
                    content.span(),
                    "unexpected tokens in attribute",
                ));
            }
            attr
        } else if ident == "repr_eq_bounds" {
            let content;
            let _paren = parenthesized!(content in input);
            let attr = Self::ReprEqBounds(Punctuated::parse_terminated(&content)?);
            if !content.is_empty() {
                return Err(syn::Error::new(
                    content.span(),
                    "unexpected tokens in attribute",
                ));
            }
            attr
        } else if ident == "repr_ord_bounds" {
            let content;
            let _paren = parenthesized!(content in input);
            let attr = Self::ReprOrdBounds(Punctuated::parse_terminated(&content)?);
            if !content.is_empty() {
                return Err(syn::Error::new(
                    content.span(),
                    "unexpected tokens in attribute",
                ));
            }
            attr
        } else if ident == "visit_repr_bounds" {
            let content;
            let _paren = parenthesized!(content in input);
            let attr = Self::VisitReprBounds(Punctuated::parse_terminated(&content)?);
            if !content.is_empty() {
                return Err(syn::Error::new(
                    content.span(),
                    "unexpected tokens in attribute",
                ));
            }
            attr
        } else if ident == "Repr" {
            input.parse::<Token![=]>()?;
            Self::Repr(input.parse()?)
        } else if ident == "repr" {
            if input.peek(Token![=]) {
                input.parse::<Token![=]>()?;
                Self::Repr(input.parse()?)
            } else {
                let content;
                let _paren = parenthesized!(content in input);
                let attr = Self::Repr(content.parse()?);
                if !content.is_empty() {
                    return Err(syn::Error::new(
                        content.span(),
                        "unexpected tokens in attribute",
                    ));
                }
                attr
            }
        } else if ident == "map" {
            if input.peek(Token![=]) {
                input.parse::<Token![=]>()?;
                Self::Map(input.parse()?)
            } else {
                let content;
                let _paren = parenthesized!(content in input);
                let attr = Self::Map(content.parse()?);
                if !content.is_empty() {
                    return Err(syn::Error::new(
                        content.span(),
                        "unexpected tokens in attribute",
                    ));
                }
                attr
            }
        } else {
            return Err(syn::Error::new(ident.span(), "unsupported attribute"));
        };
        Ok(val)
    }
}

impl Attr {
    fn parse_list(input: ParseStream<'_>) -> Result<Punctuated<Self, Token![,]>> {
        let list = Punctuated::parse_terminated(input)?;
        if !input.is_empty() {
            return Err(syn::Error::new(
                input.span(),
                "unexpected tokens in attribute",
            ));
        }
        Ok(list)
    }
}

impl Attrs {
    fn parse_all(attrs: impl IntoIterator<Item: Borrow<Attribute>>) -> Result<Self> {
        let mut crate_path = None;
        let mut bounds = None;
        let mut hash_bounds = None;
        let mut repr_eq_bounds = None;
        let mut repr_ord_bounds = None;
        let mut visit_repr_bounds = None;
        let mut repr = None;
        let mut map = None;

        for attr in attrs {
            let attr = attr.borrow();
            if attr.path().is_ident("portable") {
                let span = attr.path().get_ident().unwrap().span();
                for attr in attr.parse_args_with(Attr::parse_list)? {
                    match attr {
                        Attr::Crate(path) => {
                            if crate_path.is_some() {
                                return Err(syn::Error::new(
                                    span,
                                    "attribute specified more than once",
                                ));
                            }
                            crate_path = Some(path);
                        }
                        Attr::Bounds(b) => {
                            if bounds.is_some() {
                                return Err(syn::Error::new(
                                    span,
                                    "attribute specified more than once",
                                ));
                            }
                            bounds = Some(b);
                        }
                        Attr::HashBounds(b) => {
                            if hash_bounds.is_some() {
                                return Err(syn::Error::new(
                                    span,
                                    "attribute specified more than once",
                                ));
                            }
                            hash_bounds = Some(b);
                        }
                        Attr::ReprEqBounds(b) => {
                            if repr_eq_bounds.is_some() {
                                return Err(syn::Error::new(
                                    span,
                                    "attribute specified more than once",
                                ));
                            }
                            repr_eq_bounds = Some(b);
                        }
                        Attr::ReprOrdBounds(b) => {
                            if repr_ord_bounds.is_some() {
                                return Err(syn::Error::new(
                                    span,
                                    "attribute specified more than once",
                                ));
                            }
                            repr_ord_bounds = Some(b);
                        }
                        Attr::VisitReprBounds(b) => {
                            if visit_repr_bounds.is_some() {
                                return Err(syn::Error::new(
                                    span,
                                    "attribute specified more than once",
                                ));
                            }
                            visit_repr_bounds = Some(b);
                        }
                        Attr::Repr(r) => {
                            if repr.is_some() {
                                return Err(syn::Error::new(
                                    span,
                                    "attribute specified more than once",
                                ));
                            }
                            repr = Some(r);
                        }
                        Attr::Map(m) => {
                            if map.is_some() {
                                return Err(syn::Error::new(
                                    span,
                                    "attribute specified more than once",
                                ));
                            }
                            map = Some(m);
                        }
                    }
                }
            }
        }

        let crate_path = crate_path.unwrap_or_else(|| parse_quote!(::portable));

        Ok(Attrs {
            crate_path,
            bounds,
            hash_bounds,
            repr_eq_bounds,
            repr_ord_bounds,
            visit_repr_bounds,
            repr,
            map,
        })
    }

    fn hash_bounds(&self) -> Option<&Punctuated<WherePredicate, Token![,]>> {
        match (&self.bounds, &self.hash_bounds) {
            (_, Some(b)) => Some(b),
            (Some(b), _) => Some(b),
            _ => None,
        }
    }

    fn repr_eq_bounds(&self) -> Option<&Punctuated<WherePredicate, Token![,]>> {
        match (&self.bounds, &self.repr_eq_bounds) {
            (_, Some(b)) => Some(b),
            (Some(b), _) => Some(b),
            _ => None,
        }
    }

    fn repr_ord_bounds(&self) -> Option<&Punctuated<WherePredicate, Token![,]>> {
        match (&self.bounds, &self.repr_ord_bounds) {
            (_, Some(b)) => Some(b),
            (Some(b), _) => Some(b),
            _ => None,
        }
    }

    fn visit_repr_bounds(&self) -> Option<&Punctuated<WherePredicate, Token![,]>> {
        match (&self.bounds, &self.visit_repr_bounds) {
            (_, Some(b)) => Some(b),
            (Some(b), _) => Some(b),
            _ => None,
        }
    }
}

/// Applies the bounds for a derived impl to `generics`.
///
/// Custom bounds are additive: a type parameter's declared bounds are required by the type
/// definition itself, so they are kept and the custom predicates are merged into the where
/// clause. Without custom bounds, `default_bound` is applied to every type parameter.
fn apply_bounds(
    generics: &mut Generics,
    custom: Option<&Punctuated<WherePredicate, Token![,]>>,
    default_bound: Option<TypeParamBound>,
) {
    if let Some(custom) = custom {
        generics
            .make_where_clause()
            .predicates
            .extend(custom.iter().cloned());
    } else if let Some(bound) = default_bound {
        for param in &mut generics.params {
            if let GenericParam::Type(ty) = param {
                ty.bounds.push(bound.clone());
            }
        }
    }
}

#[proc_macro_derive(VisitPortableRepr, attributes(portable))]
pub fn derive_visit_portable_repr(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut derive = parse_macro_input!(input as DeriveInput);

    let attrs = match Attrs::parse_all(&derive.attrs) {
        Ok(attrs) => attrs,
        Err(err) => return err.into_compile_error().into(),
    };

    let crate_path = &attrs.crate_path;

    apply_bounds(&mut derive.generics, attrs.visit_repr_bounds(), None);

    let (impl_generics, type_generics, where_clause) = derive.generics.split_for_impl();
    let ty_name = &derive.ident;

    let repr = attrs.repr.clone().unwrap_or_else(|| parse_quote! { Self });

    let visit_impl = if let Some(map) = &attrs.map {
        quote! {
            #crate_path::repr::VisitPortableRepr::visit_portable_repr(
                &((#map)(self)),
                __f,
            )
        }
    } else {
        quote! { __f(self) }
    };

    quote! {
        impl #impl_generics #crate_path::repr::VisitPortableRepr for #ty_name #type_generics #where_clause {
            type Repr = #repr;

            fn visit_portable_repr<__F, __R>(&self, __f: __F) -> __R
            where
                __F: ::core::ops::FnOnce(&Self::Repr) -> __R,
            {
                #visit_impl
            }
        }
    }.into()
}

#[proc_macro_derive(PortableReprEq, attributes(portable))]
pub fn derive_portable_repr_eq(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut derive = parse_macro_input!(input as DeriveInput);

    let attrs = match Attrs::parse_all(&derive.attrs) {
        Ok(attrs) => attrs,
        Err(err) => return err.into_compile_error().into(),
    };

    let crate_path = &attrs.crate_path;

    apply_bounds(
        &mut derive.generics,
        attrs.repr_eq_bounds(),
        Some(parse_quote!(#crate_path::eq::PortableEq)),
    );

    let (impl_generics, type_generics, where_clause) = derive.generics.split_for_impl();
    let ty_name = &derive.ident;

    let eq_impl = match &derive.data {
        Data::Struct(data) => match &data.fields {
            Fields::Unit => quote! { true },
            Fields::Unnamed(fields) => {
                let exprs = (0..fields.unnamed.len()).map(|idx| {
                    let idx = Index {
                        index: idx as u32,
                        span: Span::call_site(),
                    };
                    quote! {
                        #crate_path::eq::PortableEq::portable_eq(&self.#idx, &__other.#idx)
                    }
                });
                quote! { true #(&& #exprs)* }
            }
            Fields::Named(fields) => {
                let exprs = fields.named.iter().map(|field| {
                    let name = &field.ident;
                    quote! {
                        #crate_path::eq::PortableEq::portable_eq(&self.#name, &__other.#name)
                    }
                });
                quote! { true #(&& #exprs)* }
            }
        },
        Data::Enum(data) => {
            if data.variants.is_empty() {
                quote! { true }
            } else {
                let variants = data.variants.iter().map(|variant| {
                    let name = &variant.ident;
                    match &variant.fields {
                        Fields::Unit => quote! {
                            (Self::#name, Self::#name) => true,
                        },
                        Fields::Unnamed(fields) => {
                            let field_names_l: Vec<Ident> = (0..fields.unnamed.len()).map(|idx| Ident::new(&format!("__field_l_{idx}"), Span::call_site())).collect();
                            let field_names_r: Vec<Ident> = (0..fields.unnamed.len()).map(|idx| Ident::new(&format!("__field_r_{idx}"), Span::call_site())).collect();
                            let field_names_l = field_names_l.as_slice();
                            let field_names_r = field_names_r.as_slice();
                            quote! {
                                (Self::#name(#(#field_names_l),*), Self::#name(#(#field_names_r),*)) =>
                                    true #(&& #crate_path::eq::PortableEq::portable_eq(#field_names_l, #field_names_r))*,
                            }
                        },
                        Fields::Named(fields) => {
                            let field_names: Vec<&Ident> = fields.named.iter().filter_map(|field| field.ident.as_ref()).collect();
                            let field_names_l: Vec<Ident> = field_names.iter().copied().map(|ident| Ident::new(&format!("__field_l_{ident}"), ident.span())).collect();
                            let field_names_r: Vec<Ident> = field_names.iter().copied().map(|ident| Ident::new(&format!("__field_r_{ident}"), ident.span())).collect();
                            let field_names = field_names.as_slice();
                            let field_names_l = field_names_l.as_slice();
                            let field_names_r = field_names_r.as_slice();
                            quote! {
                                (Self::#name { #(#field_names: #field_names_l),* }, Self::#name { #(#field_names: #field_names_r),* }) =>
                                    true #(&& #crate_path::eq::PortableEq::portable_eq(#field_names_l, #field_names_r))*,
                            }
                        },
                    }
                });
                quote! {
                    match (self, __other) {
                        #(#variants)*
                        #[allow(unreachable_patterns)]
                        _ => false,
                    }
                }
            }
        }
        _ => {
            return quote! {
                ::core::compile_error!("derive(PortableReprEq) is not supported for unions");
            }
            .into();
        }
    };

    quote! {
        impl #impl_generics #crate_path::eq::PortableReprEq for #ty_name #type_generics #where_clause {
            fn portable_repr_eq(&self, __other: &Self) -> bool {
                #eq_impl
            }
        }
    }.into()
}

#[proc_macro_derive(PortableReprOrd, attributes(portable))]
pub fn derive_portable_repr_ord(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut derive = parse_macro_input!(input as DeriveInput);

    let attrs = match Attrs::parse_all(&derive.attrs) {
        Ok(attrs) => attrs,
        Err(err) => return err.into_compile_error().into(),
    };

    let crate_path = &attrs.crate_path;

    apply_bounds(
        &mut derive.generics,
        attrs.repr_ord_bounds(),
        Some(parse_quote!(#crate_path::ord::PortableOrd)),
    );

    let (impl_generics, type_generics, where_clause) = derive.generics.split_for_impl();
    let ty_name = &derive.ident;

    let ord_impl = match &derive.data {
        Data::Struct(data) => match &data.fields {
            Fields::Unit => quote! { ::core::cmp::Ordering::Equal },
            Fields::Unnamed(fields) => {
                let exprs = (0..fields.unnamed.len()).map(|idx| {
                    let idx = Index {
                        index: idx as u32,
                        span: Span::call_site(),
                    };
                    quote! {
                        #crate_path::ord::PortableOrd::portable_cmp(&self.#idx, &__other.#idx)
                    }
                });
                quote! {
                    #(
                        let __ord = #exprs;
                        if ::core::cmp::Ordering::is_ne(__ord) {
                            return __ord;
                        }
                    )*
                    ::core::cmp::Ordering::Equal
                }
            }
            Fields::Named(fields) => {
                let exprs = fields.named.iter().map(|field| {
                    let name = &field.ident;
                    quote! {
                        #crate_path::ord::PortableOrd::portable_cmp(&self.#name, &__other.#name)
                    }
                });
                quote! {
                    #(
                        let __ord = #exprs;
                        if ::core::cmp::Ordering::is_ne(__ord) {
                            return __ord;
                        }
                    )*
                    ::core::cmp::Ordering::Equal
                }
            }
        },
        Data::Enum(data) => {
            if data.variants.is_empty() {
                quote! { ::core::cmp::Ordering::Equal }
            } else {
                let variants = data.variants.iter().map(|variant| {
                    let name = &variant.ident;
                    match &variant.fields {
                        Fields::Unit => quote! {
                            (Self::#name, Self::#name) => ::core::cmp::Ordering::Equal,
                            #[allow(unreachable_patterns)]
                            (Self::#name, _) => ::core::cmp::Ordering::Less,
                            #[allow(unreachable_patterns)]
                            (_, Self::#name) => ::core::cmp::Ordering::Greater,
                        },
                        Fields::Unnamed(fields) => {
                            let field_names_l: Vec<Ident> = (0..fields.unnamed.len()).map(|idx| Ident::new(&format!("__field_l_{idx}"), Span::call_site())).collect();
                            let field_names_r: Vec<Ident> = (0..fields.unnamed.len()).map(|idx| Ident::new(&format!("__field_r_{idx}"), Span::call_site())).collect();
                            let field_names_l = field_names_l.as_slice();
                            let field_names_r = field_names_r.as_slice();
                            quote! {
                                (Self::#name(#(#field_names_l),*), Self::#name(#(#field_names_r),*)) => {
                                    #(
                                        let __ord = #crate_path::ord::PortableOrd::portable_cmp(#field_names_l, #field_names_r);
                                        if ::core::cmp::Ordering::is_ne(__ord) {
                                            return __ord;
                                        }
                                    )*
                                    ::core::cmp::Ordering::Equal
                                }
                                #[allow(unreachable_patterns)]
                                (Self::#name(..), _) => ::core::cmp::Ordering::Less,
                                #[allow(unreachable_patterns)]
                                (_, Self::#name(..)) => ::core::cmp::Ordering::Greater,
                            }
                        },
                        Fields::Named(fields) => {
                            let field_names: Vec<&Ident> = fields.named.iter().filter_map(|field| field.ident.as_ref()).collect();
                            let field_names_l: Vec<Ident> = field_names.iter().copied().map(|ident| Ident::new(&format!("__field_l_{ident}"), ident.span())).collect();
                            let field_names_r: Vec<Ident> = field_names.iter().copied().map(|ident| Ident::new(&format!("__field_r_{ident}"), ident.span())).collect();
                            let field_names = field_names.as_slice();
                            let field_names_l = field_names_l.as_slice();
                            let field_names_r = field_names_r.as_slice();
                            quote! {
                                (Self::#name { #(#field_names: #field_names_l),* }, Self::#name { #(#field_names: #field_names_r),* }) => {
                                    #(
                                        let __ord = #crate_path::ord::PortableOrd::portable_cmp(#field_names_l, #field_names_r);
                                        if ::core::cmp::Ordering::is_ne(__ord) {
                                            return __ord;
                                        }
                                    )*
                                    ::core::cmp::Ordering::Equal
                                }
                                #[allow(unreachable_patterns)]
                                (Self::#name { .. }, _) => ::core::cmp::Ordering::Less,
                                #[allow(unreachable_patterns)]
                                (_, Self::#name { .. }) => ::core::cmp::Ordering::Greater,
                            }
                        },
                    }
                });
                quote! {
                    match (self, __other) {
                        #(#variants)*
                    }
                }
            }
        }
        _ => {
            return quote! {
                ::core::compile_error!("derive(PortableReprOrd) is not supported for unions");
            }
            .into();
        }
    };

    quote! {
        impl #impl_generics #crate_path::ord::PortableReprOrd for #ty_name #type_generics #where_clause {
            fn portable_repr_cmp(&self, __other: &Self) -> ::core::cmp::Ordering {
                #ord_impl
            }
        }
    }.into()
}

#[proc_macro_derive(PortableHash, attributes(portable))]
pub fn derive_portable_hash(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut derive = parse_macro_input!(input as DeriveInput);

    let attrs = match Attrs::parse_all(&derive.attrs) {
        Ok(attrs) => attrs,
        Err(err) => return err.into_compile_error().into(),
    };

    let crate_path = &attrs.crate_path;

    apply_bounds(
        &mut derive.generics,
        attrs.hash_bounds(),
        Some(parse_quote!(#crate_path::PortableHash)),
    );

    let (impl_generics, type_generics, where_clause) = derive.generics.split_for_impl();
    let ty_name = &derive.ident;

    let hash_impl = match &derive.data {
        Data::Struct(data) => match &data.fields {
            Fields::Unit => quote! {},
            Fields::Unnamed(fields) => {
                let exprs = (0..fields.unnamed.len()).map(|idx| {
                    let idx = Index {
                        index: idx as u32,
                        span: Span::call_site(),
                    };
                    quote! {
                        #crate_path::PortableHash::portable_hash(&self.#idx, __state);
                    }
                });
                quote! { #(#exprs)* }
            }
            Fields::Named(fields) => {
                let exprs = fields.named.iter().map(|field| {
                    let name = &field.ident;
                    quote! {
                        #crate_path::PortableHash::portable_hash(&self.#name, __state);
                    }
                });
                quote! { #(#exprs)* }
            }
        },
        Data::Enum(data) => {
            if data.variants.is_empty() {
                quote! {}
            } else {
                let discrim = UintType::min_for(data.variants.len());
                let variants = data.variants.iter().enumerate().map(|(idx, variant)| {
                    let name = &variant.ident;
                    let idx = discrim.lit(idx);
                    match &variant.fields {
                        Fields::Unit => quote! {
                            Self::#name => #crate_path::PortableHash::portable_hash(&#idx, __state),
                        },
                        Fields::Unnamed(fields) => {
                            let field_names: Vec<Ident> = (0..fields.unnamed.len()).map(|idx| Ident::new(&format!("__field_{idx}"), Span::call_site())).collect();
                            let field_names = field_names.as_slice();
                            quote! {
                                Self::#name(#(#field_names),*) => {
                                    #crate_path::PortableHash::portable_hash(&#idx, __state);
                                    #(#crate_path::PortableHash::portable_hash(#field_names, __state);)*
                                }
                            }
                        },
                        Fields::Named(fields) => {
                            let field_names: Vec<&Ident> = fields.named.iter().filter_map(|field| field.ident.as_ref()).collect();
                            let field_names = field_names.as_slice();
                            quote! {
                                Self::#name { #(#field_names),* } => {
                                    #crate_path::PortableHash::portable_hash(&#idx, __state);
                                    #(#crate_path::PortableHash::portable_hash(#field_names, __state);)*
                                }
                            }
                        },
                    }
                });
                quote! {
                    match self {
                        #(#variants)*
                    }
                }
            }
        }
        _ => {
            return quote! {
                ::core::compile_error!("derive(PortableHash) is not supported for unions");
            }
            .into();
        }
    };

    quote! {
        impl #impl_generics #crate_path::PortableHash for #ty_name #type_generics #where_clause {
            fn portable_hash<__H>(&self, __state: &mut __H)
            where
                __H: ::core::hash::Hasher,
            {
                #hash_impl
            }
        }
    }
    .into()
}

enum UintType {
    U8,
    U16,
    U32,
    U64,
}

impl UintType {
    fn min_for(len: usize) -> Self {
        if len <= (u8::MAX as usize) + 1 {
            Self::U8
        } else if len <= (u16::MAX as usize) + 1 {
            Self::U16
        } else if (len as u64) <= (u32::MAX as u64) + 1 {
            Self::U32
        } else {
            Self::U64
        }
    }

    fn lit(&self, n: usize) -> TokenStream {
        match self {
            Self::U8 => {
                assert!(n <= (u8::MAX as usize));
                let n = n as u8;
                quote! { #n }
            }
            Self::U16 => {
                assert!(n <= (u16::MAX as usize));
                let n = n as u16;
                quote! { #n }
            }
            Self::U32 => {
                assert!((n as u64) <= (u32::MAX as u64));
                let n = n as u32;
                quote! { #n }
            }
            Self::U64 => {
                assert!((n as u64) <= u64::MAX);
                let n = n as u64;
                quote! { #n }
            }
        }
    }
}
