extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token::Bracket,
    Expr, LitInt, Token,
};

#[proc_macro]
pub fn assert_body_matches(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as BodyMatchCall);

    let pattern_constructor = input.expand_pattern_constructor();
    let check_call = input.expand_check_call();

    proc_macro::TokenStream::from(quote! {
        #pattern_constructor
        #check_call
    })
}

impl BodyMatchCall {
    fn expand_pattern_constructor(&self) -> TokenStream {
        let pat = self.pat.expand();
        quote! {
            let pat = #pat;
        }
    }

    fn expand_check_call(&self) -> TokenStream {
        let value = &self.value;
        quote! {
            restest::__private::assert_matches(#value, pat);
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
    fn expand(&self) -> TokenStream {
        match &self.kind {
            PatternKind::Integer(i) => quote! { restest::__private::Pattern::Integer(#i) },
            PatternKind::Array(a) => {
                let elems = a.iter().map(|elem| elem.expand());

                quote! {
                    restest::__private::Pattern::Array(vec![ #( #elems ),* ])
                }
            }
        }
    }
}

impl Parse for Pattern {
    fn parse(input: ParseStream) -> syn::Result<Pattern> {
        let kind = if input.peek(LitInt) {
            PatternKind::Integer(input.parse()?)
        } else if input.peek(Bracket) {
            let inner;
            let _ = bracketed!(inner in input);
            let elems = Punctuated::parse_terminated(&inner)?;

            PatternKind::Array(elems)
        } else {
            return Err(input.error("Excepted an integer or `[`"));
        };

        Ok(Pattern { kind })
    }
}

enum PatternKind {
    Array(Punctuated<Pattern, Token![,]>),
    Integer(LitInt),
}
