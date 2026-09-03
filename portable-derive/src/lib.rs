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
    /// `repr`, `repr = T` or `repr(T)`. Both forms are accepted for both meanings; `map`
    /// decides which. `None` is a bare `repr`.
    Repr(Option<Type>),
    Map(Expr),
    Rkyv(Option<Ident>),
    RkyvCrate(Path),
}

/// What a type's representation is, resolved from the `repr` and `map` attributes.
///
/// `repr` names the representation in either syntax; `map` decides the meaning. With a `map`
/// the representation is an existing type that the map reaches, and without one it is
/// generated, because a type can only hand out a reference to itself.
enum ReprKind {
    /// The type is its own representation: the default, or an explicit `repr = Self`.
    SelfRepr,
    /// The representation is an existing type, reached by `map`.
    Delegate(Box<Type>),
    /// The representation is generated, optionally under a given name.
    Generate(Option<Ident>),
}

struct Attrs {
    crate_path: Path,
    bounds: Option<Punctuated<WherePredicate, Token![,]>>,
    hash_bounds: Option<Punctuated<WherePredicate, Token![,]>>,
    repr_eq_bounds: Option<Punctuated<WherePredicate, Token![,]>>,
    repr_ord_bounds: Option<Punctuated<WherePredicate, Token![,]>>,
    visit_repr_bounds: Option<Punctuated<WherePredicate, Token![,]>>,
    repr: ReprKind,
    map: Option<Expr>,
    rkyv: Option<Option<Ident>>,
    rkyv_crate: Option<Path>,
    /// The archived type named by rkyv's own `#[rkyv(archived = ...)]`, if the type carries one.
    rkyv_archived: Option<Ident>,
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
        } else if ident == "repr" || ident == "Repr" {
            if input.peek(Token![=]) {
                input.parse::<Token![=]>()?;
                Self::Repr(Some(input.parse()?))
            } else if input.peek(syn::token::Paren) {
                let content;
                let _paren = parenthesized!(content in input);
                let attr = Self::Repr(Some(content.parse()?));
                if !content.is_empty() {
                    return Err(syn::Error::new(
                        content.span(),
                        "unexpected tokens in attribute",
                    ));
                }
                attr
            } else {
                Self::Repr(None)
            }
        } else if ident == "rkyv_crate" {
            if input.peek(Token![=]) {
                input.parse::<Token![=]>()?;
                Self::RkyvCrate(input.parse()?)
            } else {
                let content;
                let _paren = parenthesized!(content in input);
                let attr = Self::RkyvCrate(content.parse()?);
                if !content.is_empty() {
                    return Err(syn::Error::new(
                        content.span(),
                        "unexpected tokens in attribute",
                    ));
                }
                attr
            }
        } else if ident == "rkyv" {
            let name = if input.peek(Token![=]) {
                input.parse::<Token![=]>()?;
                Some(input.parse()?)
            } else if input.peek(syn::token::Paren) {
                let content;
                let _paren = parenthesized!(content in input);
                let name = content.parse()?;
                if !content.is_empty() {
                    return Err(syn::Error::new(
                        content.span(),
                        "unexpected tokens in attribute",
                    ));
                }
                Some(name)
            } else {
                None
            };
            Self::Rkyv(name)
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
        let mut rkyv = None;
        let mut rkyv_crate = None;
        let mut rkyv_archived = None;

        for attr in attrs {
            let attr = attr.borrow();
            // rkyv's own attribute names the archived type, so the archived impls do not need
            // that name repeated in a `portable` attribute.
            if attr.path().is_ident("rkyv") {
                let _ = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("archived") {
                        if let Ok(value) = meta.value() {
                            rkyv_archived = value.parse().ok();
                        }
                    } else {
                        // Consume any value so unrelated rkyv options do not abort the walk.
                        let _ = meta.value().and_then(|v| v.parse::<Expr>());
                    }
                    Ok(())
                });
            } else if attr.path().is_ident("portable") {
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
                        Attr::Rkyv(name) => {
                            if rkyv.is_some() {
                                return Err(syn::Error::new(
                                    span,
                                    "attribute specified more than once",
                                ));
                            }
                            rkyv = Some(name);
                        }
                        Attr::RkyvCrate(path) => {
                            if rkyv_crate.is_some() {
                                return Err(syn::Error::new(
                                    span,
                                    "attribute specified more than once",
                                ));
                            }
                            rkyv_crate = Some(path);
                        }
                    }
                }
            }
        }

        let crate_path = crate_path.unwrap_or_else(|| parse_quote!(::portable));

        // Archived impls exist to share a generated representation, so they cannot be
        // combined with one reached by `map`.
        if let (Some(_), Some(map)) = (&rkyv, &map) {
            return Err(syn::Error::new_spanned(
                map,
                "`rkyv` generates a representation and cannot be combined with `map`, \
                 which reaches an existing one",
            ));
        }

        let repr = match (repr, &map) {
            // An explicit `Self` is the default spelled out, whether or not a map is present.
            (Some(Some(ty)), _) if is_self_type(&ty) => ReprKind::SelfRepr,
            // With a map the representation is whatever the map reaches.
            (repr, Some(_)) => ReprKind::Delegate(Box::new(
                repr.flatten().unwrap_or_else(|| parse_quote!(Self)),
            )),
            // Otherwise naming a representation asks for one to be generated under that name.
            (Some(Some(ty)), None) => {
                let Some(name) = type_as_ident(&ty) else {
                    return Err(syn::Error::new_spanned(
                        &ty,
                        "expected a single identifier naming the representation to generate, \
                         or a `map` reaching an existing one",
                    ));
                };
                ReprKind::Generate(Some(name))
            }
            (Some(None), None) => ReprKind::Generate(None),
            (None, None) if rkyv.is_some() => ReprKind::Generate(None),
            (None, None) => ReprKind::SelfRepr,
        };

        if rkyv.is_some() && !matches!(repr, ReprKind::Generate(_)) {
            return Err(syn::Error::new(
                Span::call_site(),
                "`rkyv` requires a generated representation; remove `repr = Self`",
            ));
        }

        // A generated representation carries the comparison impls itself, over the field
        // types rather than over the type's own parameters, so per-derive comparison bounds
        // would have nothing to apply to. Reject them rather than ignore them.
        if matches!(repr, ReprKind::Generate(_)) {
            for (bounds, name) in [
                (&repr_eq_bounds, "repr_eq_bounds"),
                (&repr_ord_bounds, "repr_ord_bounds"),
            ] {
                if let Some(bounds) = bounds {
                    return Err(syn::Error::new_spanned(
                        bounds,
                        format!(
                            "`{name}` has no effect on a generated representation, whose \
                             comparison impl is bounded by the field types instead; use \
                             `repr = Self` or `repr = ..., map = ...` to bound the impl on \
                             the type itself",
                        ),
                    ));
                }
            }
        }

        // The rkyv crate path is only used by the archived impls.
        if rkyv.is_none()
            && let Some(path) = &rkyv_crate
        {
            return Err(syn::Error::new_spanned(
                path,
                "`rkyv_crate` has no effect without `rkyv`, which requests the archived impls",
            ));
        }

        Ok(Attrs {
            crate_path,
            bounds,
            hash_bounds,
            repr_eq_bounds,
            repr_ord_bounds,
            visit_repr_bounds,
            repr,
            map,
            rkyv,
            rkyv_crate,
            rkyv_archived,
        })
    }

    /// The path to the `rkyv` crate, which the archived impls are written against.
    fn rkyv_path(&self) -> Path {
        self.rkyv_crate
            .clone()
            .unwrap_or_else(|| parse_quote!(::rkyv))
    }

    /// Returns the name of the generated representation, if one is to be generated.
    fn gen_repr_name(&self, ty_name: &Ident) -> Option<Ident> {
        let ReprKind::Generate(explicit) = &self.repr else {
            return None;
        };
        Some(
            explicit
                .clone()
                .unwrap_or_else(|| Ident::new(&format!("{ty_name}Repr"), Span::call_site())),
        )
    }

    /// Returns the name of the archived counterpart, if archived impls were requested.
    ///
    /// Prefers an explicit `#[portable(rkyv = ...)]`, then rkyv's own
    /// `#[rkyv(archived = ...)]`, then rkyv's default naming.
    fn archived_name(&self, ty_name: &Ident) -> Option<Ident> {
        let explicit = self.rkyv.as_ref()?;
        Some(
            explicit
                .clone()
                .or_else(|| self.rkyv_archived.clone())
                .unwrap_or_else(|| Ident::new(&format!("Archived{ty_name}"), Span::call_site())),
        )
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

/// A generated representation: a view of a type's fields that erases both their types and the
/// lifetime of the borrow.
///
/// Every type that visits to the same shim compares against every other, in both directions,
/// through the blanket [`PortableEq`] impl — which is how a native type and its archived
/// counterpart interoperate without either naming the other.
struct Shim<'a> {
    name: Ident,
    /// One generic parameter per field, in declaration order.
    params: Vec<Ident>,
    /// The declared type of each field, positionally matching `params`.
    field_types: Vec<&'a Type>,
    data: &'a Data,
}

impl<'a> Shim<'a> {
    fn new(name: Ident, data: &'a Data) -> Self {
        let fields: Vec<&Type> = shim_field_types(data);
        let params = (0..fields.len())
            .map(|idx| Ident::new(&format!("__T{idx}"), Span::call_site()))
            .collect();
        Self {
            name,
            params,
            field_types: fields,
            data,
        }
    }

    /// Renders `<A, B>`, or nothing when the type has no fields.
    fn args(&self, args: &[TokenStream]) -> TokenStream {
        if args.is_empty() {
            quote! {}
        } else {
            quote! { <#(#args),*> }
        }
    }

    /// Renders the shim's own parameter list, optionally bounded.
    fn decl(&self, params: &[Ident]) -> TokenStream {
        let args: Vec<TokenStream> = params.iter().map(|p| quote!(#p: ?Sized)).collect();
        self.args(&args)
    }

    /// Renders `Shim<A, B>` for the given arguments.
    fn path(&self, args: &[TokenStream]) -> TokenStream {
        let name = &self.name;
        let args = self.args(args);
        quote! { #name #args }
    }

    fn param_args(&self) -> Vec<TokenStream> {
        self.params.iter().map(|p| quote!(#p)).collect()
    }

    /// The shim instantiated with the native field types.
    fn native_args(&self) -> Vec<TokenStream> {
        self.field_types.iter().map(|ty| quote!(#ty)).collect()
    }

    /// The shim instantiated with the archived counterparts of the field types.
    fn archived_args(&self, rkyv: &Path) -> Vec<TokenStream> {
        self.field_types
            .iter()
            .map(|ty| quote!(<#ty as #rkyv::Archive>::Archived))
            .collect()
    }

    /// The shim's type definition.
    fn definition(&self, ty_name: &Ident, vis: &syn::Visibility, crate_path: &Path) -> TokenStream {
        let name = &self.name;
        let decl = self.decl(&self.params);
        let field = |idx: usize| {
            let param = &self.params[idx];
            quote! { #crate_path::repr::Field<#param> }
        };
        let doc = format!(
            "Portable representation of [`{ty_name}`], holding a borrowed view of each field.\n\n             Every type visiting to this representation compares against every other, in both              directions, so a value and its archived counterpart interoperate without either              naming the other.",
        );

        match self.data {
            Data::Struct(data) => {
                let body = match &data.fields {
                    Fields::Unit => quote! { ; },
                    Fields::Unnamed(fields) => {
                        let entries = (0..fields.unnamed.len()).map(|idx| {
                            let ty = field(idx);
                            quote! { #vis #ty }
                        });
                        quote! { ( #(#entries),* ); }
                    }
                    Fields::Named(fields) => {
                        let entries = fields.named.iter().enumerate().map(|(idx, f)| {
                            let name = &f.ident;
                            let ty = field(idx);
                            quote! { #vis #name: #ty }
                        });
                        quote! { { #(#entries),* } }
                    }
                };
                quote! {
                    #[doc = #doc]
                    #vis struct #name #decl #body
                }
            }
            Data::Enum(data) => {
                let mut idx = 0;
                let variants = data.variants.iter().map(|variant| {
                    let vname = &variant.ident;
                    match &variant.fields {
                        Fields::Unit => quote! { #vname, },
                        Fields::Unnamed(fields) => {
                            let entries: Vec<TokenStream> = fields
                                .unnamed
                                .iter()
                                .map(|_| {
                                    let ty = field(idx);
                                    idx += 1;
                                    ty
                                })
                                .collect();
                            quote! { #vname(#(#entries),*), }
                        }
                        Fields::Named(fields) => {
                            let entries: Vec<TokenStream> = fields
                                .named
                                .iter()
                                .map(|f| {
                                    let name = &f.ident;
                                    let ty = field(idx);
                                    idx += 1;
                                    quote! { #name: #ty }
                                })
                                .collect();
                            quote! { #vname { #(#entries),* }, }
                        }
                    }
                });
                quote! {
                    #[doc = #doc]
                    #vis enum #name #decl { #(#variants)* }
                }
            }
            _ => quote! {},
        }
    }

    /// An expression building the shim from `self`, for use inside `visit_portable_repr`.
    ///
    /// Field names and variant shapes are identical between a type and its archived
    /// counterpart, so this expression serves both impls unchanged.
    fn build(&self) -> TokenStream {
        let name = &self.name;
        let new = quote! { __field };

        match self.data {
            Data::Struct(data) => match &data.fields {
                Fields::Unit => quote! { #name },
                Fields::Unnamed(fields) => {
                    let entries = (0..fields.unnamed.len()).map(|idx| {
                        let idx = Index {
                            index: idx as u32,
                            span: Span::call_site(),
                        };
                        quote! { #new(&self.#idx) }
                    });
                    quote! { #name(#(#entries),*) }
                }
                Fields::Named(fields) => {
                    let entries = fields.named.iter().map(|f| {
                        let fname = &f.ident;
                        quote! { #fname: #new(&self.#fname) }
                    });
                    quote! { #name { #(#entries),* } }
                }
            },
            Data::Enum(data) => {
                let arms = data.variants.iter().map(|variant| {
                    let vname = &variant.ident;
                    match &variant.fields {
                        Fields::Unit => quote! { Self::#vname => #name::#vname, },
                        Fields::Unnamed(fields) => {
                            let binds = binding_idents(fields.unnamed.len(), "__f");
                            quote! {
                                Self::#vname(#(#binds),*) => #name::#vname(#(#new(#binds)),*),
                            }
                        }
                        Fields::Named(fields) => {
                            let names: Vec<&Ident> = fields
                                .named
                                .iter()
                                .filter_map(|f| f.ident.as_ref())
                                .collect();
                            quote! {
                                Self::#vname { #(#names),* } =>
                                    #name::#vname { #(#names: #new(#names)),* },
                            }
                        }
                    }
                });
                quote! { match self { #(#arms)* } }
            }
            _ => quote! {},
        }
    }

    /// A `VisitPortableRepr` impl body visiting the shim built from `self`.
    fn visit_body(&self, crate_path: &Path) -> TokenStream {
        // An empty enum has no values, so this body is unreachable. `match *self {}` is the
        // only form that type-checks: a *reference* to an uninhabited type is inhabited, so
        // matching `self` rather than `*self` is not exhaustive.
        if matches!(self.data, Data::Enum(data) if data.variants.is_empty()) {
            return quote! { match *self {} };
        }

        let build = self.build();
        quote! {
            /// Reinterprets a borrow as a representation field.
            ///
            /// # Safety
            ///
            /// The returned `Field` must not outlive `value`.
            #[inline(always)]
            unsafe fn __field<__T: ?Sized>(value: &__T) -> #crate_path::repr::Field<__T> {
                // SAFETY: `Field<T>` is `#[repr(transparent)]` over `NonNull<T>`, so the two
                // have identical layout and this reads one as the other.
                unsafe {
                    *(&::core::ptr::NonNull::from_ref(value)
                        as *const ::core::ptr::NonNull<__T>)
                        .cast::<#crate_path::repr::Field<__T>>()
                }
            }

            // SAFETY: every borrow is taken from `self`, which outlives this call, and the
            // representation is lent out by reference for the duration of `__f` only, so no
            // `Field` can outlive the value it borrows.
            let __repr = unsafe { #build };
            __f(&__repr)
        }
    }

    /// Field accessor expressions for a struct shim, paired between `self` and `__other`.
    fn struct_fields(&self, fields: &Fields) -> Vec<(TokenStream, TokenStream)> {
        match fields {
            Fields::Unit => Vec::new(),
            Fields::Unnamed(f) => (0..f.unnamed.len())
                .map(|idx| {
                    let idx = Index {
                        index: idx as u32,
                        span: Span::call_site(),
                    };
                    (quote! { &*self.#idx }, quote! { &*__other.#idx })
                })
                .collect(),
            Fields::Named(f) => f
                .named
                .iter()
                .map(|f| {
                    let name = &f.ident;
                    (quote! { &*self.#name }, quote! { &*__other.#name })
                })
                .collect(),
        }
    }

    /// Destructuring patterns for one shim variant, binding `self` and `__other` fields.
    fn variant_patterns(
        &self,
        variant: &syn::Variant,
    ) -> (TokenStream, TokenStream, Vec<(TokenStream, TokenStream)>) {
        let name = &self.name;
        let vname = &variant.ident;
        match &variant.fields {
            Fields::Unit => (
                quote! { #name::#vname },
                quote! { #name::#vname },
                Vec::new(),
            ),
            Fields::Unnamed(fields) => {
                let l = binding_idents(fields.unnamed.len(), "__l");
                let r = binding_idents(fields.unnamed.len(), "__r");
                let pairs = l
                    .iter()
                    .zip(&r)
                    .map(|(l, r)| (quote! { &**#l }, quote! { &**#r }))
                    .collect();
                (
                    quote! { #name::#vname(#(#l),*) },
                    quote! { #name::#vname(#(#r),*) },
                    pairs,
                )
            }
            Fields::Named(fields) => {
                let names: Vec<&Ident> = fields
                    .named
                    .iter()
                    .filter_map(|f| f.ident.as_ref())
                    .collect();
                let l = prefixed_idents(&names, "__l");
                let r = prefixed_idents(&names, "__r");
                let pairs = l
                    .iter()
                    .zip(&r)
                    .map(|(l, r)| (quote! { &**#l }, quote! { &**#r }))
                    .collect();
                (
                    quote! { #name::#vname { #(#names: #l),* } },
                    quote! { #name::#vname { #(#names: #r),* } },
                    pairs,
                )
            }
        }
    }

    /// The body of `portable_repr_eq` comparing two shims field by field.
    fn eq_body(&self, crate_path: &Path) -> TokenStream {
        let eq = |(l, r): &(TokenStream, TokenStream)| {
            quote! { #crate_path::eq::PortableEq::portable_eq(#l, #r) }
        };
        match self.data {
            Data::Struct(data) => {
                let pairs = self.struct_fields(&data.fields);
                let exprs = pairs.iter().map(eq);
                quote! { true #(&& #exprs)* }
            }
            Data::Enum(data) => {
                if data.variants.is_empty() {
                    return quote! { true };
                }
                let arms = data.variants.iter().map(|variant| {
                    let (lpat, rpat, pairs) = self.variant_patterns(variant);
                    let exprs = pairs.iter().map(eq);
                    quote! { (#lpat, #rpat) => true #(&& #exprs)*, }
                });
                quote! {
                    match (self, __other) {
                        #(#arms)*
                        #[allow(unreachable_patterns)]
                        _ => false,
                    }
                }
            }
            _ => quote! {},
        }
    }

    /// The body of `portable_repr_cmp` comparing two shims field by field.
    ///
    /// Variants order by declaration, matching the derive on the type itself.
    fn ord_body(&self, crate_path: &Path) -> TokenStream {
        let name = &self.name;
        let cmp = |pairs: &[(TokenStream, TokenStream)]| {
            let steps = pairs.iter().map(|(l, r)| {
                quote! {
                    let __ord = #crate_path::ord::PortableOrd::portable_cmp(#l, #r);
                    if ::core::cmp::Ordering::is_ne(__ord) {
                        return __ord;
                    }
                }
            });
            quote! {
                #(#steps)*
                ::core::cmp::Ordering::Equal
            }
        };
        match self.data {
            Data::Struct(data) => cmp(&self.struct_fields(&data.fields)),
            Data::Enum(data) => {
                if data.variants.is_empty() {
                    return quote! { ::core::cmp::Ordering::Equal };
                }
                let arms = data.variants.iter().map(|variant| {
                    let (lpat, rpat, pairs) = self.variant_patterns(variant);
                    let body = cmp(&pairs);
                    let vname = &variant.ident;
                    let wild = match &variant.fields {
                        Fields::Unit => quote! { #name::#vname },
                        Fields::Unnamed(_) => quote! { #name::#vname(..) },
                        Fields::Named(_) => quote! { #name::#vname { .. } },
                    };
                    quote! {
                        (#lpat, #rpat) => { #body }
                        #[allow(unreachable_patterns)]
                        (#wild, _) => ::core::cmp::Ordering::Less,
                        #[allow(unreachable_patterns)]
                        (_, #wild) => ::core::cmp::Ordering::Greater,
                    }
                });
                quote! {
                    match (self, __other) {
                        #(#arms)*
                    }
                }
            }
            _ => quote! {},
        }
    }

    /// Parameters for a cross-shim impl: `<__T0: Bound<__U0> + ?Sized, __U0: ?Sized>`.
    fn cross_generics(
        &self,
        other: &[Ident],
        bound: &dyn Fn(&Ident) -> TokenStream,
    ) -> TokenStream {
        if self.params.is_empty() {
            return quote! {};
        }
        let lhs = self.params.iter().zip(other).map(|(t, u)| {
            let b = bound(u);
            quote! { #t: #b + ?Sized }
        });
        let rhs = other.iter().map(|u| quote! { #u: ?Sized });
        quote! { <#(#lhs,)* #(#rhs),*> }
    }
}

/// The declared type of every field, across all variants for an enum.
fn shim_field_types(data: &Data) -> Vec<&Type> {
    fn of_fields(fields: &Fields) -> Vec<&Type> {
        match fields {
            Fields::Unit => Vec::new(),
            Fields::Unnamed(f) => f.unnamed.iter().map(|f| &f.ty).collect(),
            Fields::Named(f) => f.named.iter().map(|f| &f.ty).collect(),
        }
    }
    match data {
        Data::Struct(data) => of_fields(&data.fields),
        Data::Enum(data) => data
            .variants
            .iter()
            .flat_map(|v| of_fields(&v.fields))
            .collect(),
        _ => Vec::new(),
    }
}

fn binding_idents(count: usize, prefix: &str) -> Vec<Ident> {
    (0..count)
        .map(|idx| Ident::new(&format!("{prefix}{idx}"), Span::call_site()))
        .collect()
}

fn prefixed_idents(names: &[&Ident], prefix: &str) -> Vec<Ident> {
    names
        .iter()
        .map(|name| Ident::new(&format!("{prefix}_{name}"), name.span()))
        .collect()
}

/// Adds `bound` to every type parameter, for an impl on the archived counterpart.
fn archived_generics(generics: &Generics, rkyv: &Path) -> Generics {
    let mut generics = generics.clone();
    for param in &mut generics.params {
        if let GenericParam::Type(ty) = param {
            ty.bounds.push(parse_quote!(#rkyv::Archive));
        }
    }
    generics
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

    // A shim replaces both the representation and the way it is reached, and additionally
    // emits the shim's own definition and the archived counterpart's impl.
    if let Some(shim_name) = attrs.gen_repr_name(ty_name) {
        let shim = Shim::new(shim_name, &derive.data);
        let params = shim.param_args();
        let decl = shim.decl(&shim.params);
        let shim_ty = shim.path(&params);
        let definition = shim.definition(ty_name, &derive.vis, crate_path);
        let visit_body = shim.visit_body(crate_path);
        let native_repr = shim.path(&shim.native_args());

        let archived_impl = attrs.archived_name(ty_name).map(|archived| {
            let rkyv = attrs.rkyv_path();
            let archived_generics = archived_generics(&derive.generics, &rkyv);
            let (archived_impl_generics, _, archived_where) = archived_generics.split_for_impl();
            let archived_repr = shim.path(&shim.archived_args(&rkyv));
            quote! {
                impl #archived_impl_generics #crate_path::repr::VisitPortableRepr
                    for #archived #type_generics #archived_where
                {
                    type Repr = #archived_repr;

                    fn visit_portable_repr<__F, __R>(&self, __f: __F) -> __R
                    where
                        __F: ::core::ops::FnOnce(&Self::Repr) -> __R,
                    {
                        #visit_body
                    }
                }
            }
        });

        return quote! {
            #definition

            impl #decl #crate_path::repr::VisitPortableRepr for #shim_ty {
                type Repr = Self;

                fn visit_portable_repr<__F, __R>(&self, __f: __F) -> __R
                where
                    __F: ::core::ops::FnOnce(&Self::Repr) -> __R,
                {
                    __f(self)
                }
            }

            impl #impl_generics #crate_path::repr::VisitPortableRepr
                for #ty_name #type_generics #where_clause
            {
                type Repr = #native_repr;

                fn visit_portable_repr<__F, __R>(&self, __f: __F) -> __R
                where
                    __F: ::core::ops::FnOnce(&Self::Repr) -> __R,
                {
                    #visit_body
                }
            }

            #archived_impl
        }
        .into();
    }

    let repr: Type = match &attrs.repr {
        ReprKind::Delegate(ty) => (**ty).clone(),
        _ => parse_quote! { Self },
    };

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

    // With a shim the representation is the shim, so the impl belongs on it rather than on
    // the type. One impl over both sides' parameters covers every pair of types sharing it,
    // which is what makes native and archived values compare in both directions.
    if let Some(shim_name) = attrs.gen_repr_name(&derive.ident) {
        let shim = Shim::new(shim_name, &derive.data);
        let other = binding_idents(shim.params.len(), "__U");
        let generics = shim.cross_generics(&other, &|u| {
            quote! { #crate_path::eq::PortableEq<#u> }
        });
        let self_ty = shim.path(&shim.param_args());
        let other_ty = shim.path(&other.iter().map(|u| quote!(#u)).collect::<Vec<_>>());
        let body = shim.eq_body(crate_path);

        return quote! {
            impl #generics #crate_path::eq::PortableReprEq<#other_ty> for #self_ty {
                fn portable_repr_eq(&self, __other: &#other_ty) -> bool {
                    #body
                }
            }
        }
        .into();
    }

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

    if let Some(shim_name) = attrs.gen_repr_name(&derive.ident) {
        let shim = Shim::new(shim_name, &derive.data);
        let other = binding_idents(shim.params.len(), "__U");
        let generics = shim.cross_generics(&other, &|u| {
            quote! { #crate_path::ord::PortableOrd<#u> }
        });
        let self_ty = shim.path(&shim.param_args());
        let other_ty = shim.path(&other.iter().map(|u| quote!(#u)).collect::<Vec<_>>());
        let body = shim.ord_body(crate_path);

        return quote! {
            impl #generics #crate_path::ord::PortableReprOrd<#other_ty> for #self_ty {
                fn portable_repr_cmp(&self, __other: &#other_ty) -> ::core::cmp::Ordering {
                    #body
                }
            }
        }
        .into();
    }

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

    // Captured before the native bounds are applied: the archived impl needs bounds on the
    // archived field types, not on the type parameters themselves.
    let plain_generics = derive.generics.clone();

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

    // Field names and variant shapes are identical on the archived type, so the same body
    // hashes both — which is what keeps a value and its archived form hashing equally, as the
    // crate's contract requires of values that compare equal.
    let archived_impl = attrs.archived_name(ty_name).map(|archived| {
        let rkyv = attrs.rkyv_path();
        let mut generics = archived_generics(&plain_generics, &rkyv);
        let predicates = &mut generics.make_where_clause().predicates;
        for param in &plain_generics.params {
            if let GenericParam::Type(ty) = param {
                let ident = &ty.ident;
                predicates.push(parse_quote! {
                    <#ident as #rkyv::Archive>::Archived: #crate_path::PortableHash
                });
            }
        }
        let (archived_impl_generics, _, archived_where) = generics.split_for_impl();
        quote! {
            impl #archived_impl_generics #crate_path::PortableHash
                for #archived #type_generics #archived_where
            {
                fn portable_hash<__H>(&self, __state: &mut __H)
                where
                    __H: ::core::hash::Hasher,
                {
                    #hash_impl
                }
            }
        }
    });

    quote! {
        impl #impl_generics #crate_path::PortableHash for #ty_name #type_generics #where_clause {
            fn portable_hash<__H>(&self, __state: &mut __H)
            where
                __H: ::core::hash::Hasher,
            {
                #hash_impl
            }
        }

        #archived_impl
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

/// Whether a type is written exactly `Self`.
fn is_self_type(ty: &Type) -> bool {
    type_as_ident(ty).is_some_and(|ident| ident == "Self")
}

/// A type written as a single identifier, which is how a representation to generate is named.
///
/// `syn` applies no type system, so any non-keyword identifier parses as a `Type`; this
/// recovers the identifier so it can name a generated item.
fn type_as_ident(ty: &Type) -> Option<Ident> {
    match ty {
        Type::Path(path) if path.qself.is_none() => path.path.get_ident().cloned(),
        _ => None,
    }
}
