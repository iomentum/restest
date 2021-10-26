extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token::{As, Brace, Bracket},
    Expr, Ident, LitInt, LitStr, Token, Type,
};

#[proc_macro]
pub fn assert_body_matches(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as BodyMatchCall);

    let deserialization = input.expand_value_deserialization();
    let pattern_constructor = input.expand_pattern_constructor();
    let check_call = input.expand_check_call();
    let bindings = input.expand_bindings();

    proc_macro::TokenStream::from(quote! {
        #deserialization
        #pattern_constructor
        #check_call
        #bindings
    })
}

impl BodyMatchCall {
    fn expand_value_deserialization(&self) -> TokenStream {
        let value = &self.value;

        quote! {
            let value = #value.json().await;
        }
    }

    fn expand_pattern_constructor(&self) -> TokenStream {
        let pat = self.pat.expand_matching_pattern();
        quote! {
            let pat = #pat;
        }
    }

    fn expand_check_call(&self) -> TokenStream {
        quote! {
            restest::__private::assert_matches(value.clone(), pat);
        }
    }

    fn expand_bindings(&self) -> TokenStream {
        let (names, exprs): (Vec<_>, Vec<_>) = self
            .pat
            .expand_bindings(quote! { value })
            .into_iter()
            .unzip();

        quote! {
            use restest::__private::ValueExt as _;

            #( let #names = #exprs; )*
        }
    }
}

impl Parse for BodyMatchCall {
    fn parse(input: ParseStream) -> syn::Result<BodyMatchCall> {
        let value = input.parse()?;
        let _ = input.parse::<Token![,]>()?;
        let pat = input.parse()?;

        Ok(BodyMatchCall { value, pat })
    }
}

struct BodyMatchCall {
    value: Expr,
    pat: Pattern,
}

struct Pattern {
    kind: PatternKind,
}

impl Pattern {
    fn expand_matching_pattern(&self) -> TokenStream {
        match &self.kind {
            PatternKind::Integer(i) => quote! { restest::__private::Pattern::Integer(#i) },

            PatternKind::SimpleBinding { .. } => quote! { restest::__private::Pattern::Any },

            PatternKind::String(s) => {
                quote! { restest::__private::Pattern::String(#s) }
            }

            PatternKind::UntypedObject(fields) => {
                let (fields, patterns): (Vec<_>, Vec<_>) = fields
                    .iter()
                    .map(|PatternField { name, sub_pattern }| {
                        (name, sub_pattern.expand_matching_pattern())
                    })
                    .unzip();

                quote! {
                    restest::__private::Pattern::object_from_array( [ #( (stringify!(#fields), #patterns) ),* ] )
                }
            }

            PatternKind::Array(a) => {
                let elems = a.iter().map(|elem| elem.expand_matching_pattern());

                quote! {
                    restest::__private::Pattern::Array(vec![ #( #elems ),* ])
                }
            }
        }
    }

    fn expand_bindings(&self, previous: TokenStream) -> Vec<(Ident, TokenStream)> {
        match &self.kind {
            PatternKind::Array(array) => array
                .iter()
                .enumerate()
                .flat_map(|(idx, sub_pattern)| {
                    sub_pattern.expand_bindings(quote! { #previous.to_array().get(#idx).unwrap() })
                })
                .collect(),

            PatternKind::Integer(_) => Vec::new(),

            PatternKind::SimpleBinding { name, ty } => {
                let final_call = ty.expand_final_call();
                vec![(name.clone(), quote! { #previous #final_call })]
            }

            PatternKind::String(_) => Vec::new(),

            PatternKind::UntypedObject(fields) => fields
                .iter()
                .flat_map(|PatternField { name, sub_pattern }| {
                    sub_pattern.expand_bindings(
                        quote! { #previous.to_object().get(stringify!(#name)).unwrap() },
                    )
                })
                .collect(),
        }
    }
}

impl Parse for Pattern {
    fn parse(input: ParseStream) -> syn::Result<Pattern> {
        let kind = if input.peek(LitInt) {
            PatternKind::Integer(input.parse()?)
        } else if input.peek(LitStr) {
            PatternKind::String(input.parse()?)
        } else if input.peek(Bracket) {
            let inner;
            let _ = bracketed!(inner in input);
            let elems = Punctuated::parse_terminated(&inner)?;

            PatternKind::Array(elems)
        } else if input.peek(Brace) {
            let inner;
            let _ = braced!(inner in input);
            let elems = Punctuated::parse_terminated(&inner)?;

            PatternKind::UntypedObject(elems)
        } else if input.peek(Ident) {
            let name = input.parse().unwrap();

            input.parse::<As>()?;
            let ty = input.parse()?;

            PatternKind::SimpleBinding { name, ty }
        } else {
            return Err(input.error("Excepted an integer, a string, an identifier, `{` or `[`"));
        };

        Ok(Pattern { kind })
    }
}

enum PatternKind {
    Array(Punctuated<Pattern, Token![,]>),
    Integer(LitInt),
    UntypedObject(Punctuated<PatternField, Token![,]>),
    /// Simple binding aka binding with no subpattern to match.
    SimpleBinding {
        name: Ident,
        ty: Ty,
    },
    String(LitStr),
}

struct PatternField {
    // TODO: handle destructuring such as `{ a, b, c }`.
    // This can currently be expressed with `{ a: a, b: b, c: c }`.
    name: Ident,
    sub_pattern: Pattern,
}

impl Parse for PatternField {
    fn parse(input: ParseStream) -> syn::Result<PatternField> {
        let name = input.parse()?;
        let _ = input.parse::<Token![:]>()?;
        let sub_pattern = input.parse()?;

        Ok(PatternField { name, sub_pattern })
    }
}

enum Ty {
    Array(Type),
    Deserializable(Type),
}

impl Parse for Ty {
    fn parse(input: ParseStream) -> syn::Result<Ty> {
        if input.peek(Bracket) {
            let inner;
            let _ = bracketed!(inner in input);
            inner.parse().map(Ty::Array)
        } else {
            input.parse().map(Ty::Deserializable)
        }
    }
}

impl Ty {
    fn expand_final_call(&self) -> TokenStream {
        match self {
            Ty::Array(inner_ty) => quote! { .to_owned().deserialize::<Vec<#inner_ty>>() },
            Ty::Deserializable(ty) => quote! { .to_owned().deserialize::<#ty>() },
        }
    }
}
