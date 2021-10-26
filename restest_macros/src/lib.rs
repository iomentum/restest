extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    visit_mut::{self, VisitMut},
    Expr, Ident, Pat, PatIdent, PatLit, Token,
};

#[proc_macro]
pub fn assert_body_matches(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as BodyMatchCall);

    proc_macro::TokenStream::from(input.expand())
}

impl BodyMatchCall {
    fn expand(mut self) -> TokenStream {
        // We need to do two things:
        //   - keep track of the variables we capture in the pattern, so that we
        //     can add them to the scope,
        //   - change each literal pattern to a binding that is checked in a
        //     separate guard.
        let PatternVisitor {
            bound_variables,
            checked_variables,
        } = PatternVisitor::from_pattern(&mut self.pat);

        let value = self.value;
        let pat = self.pat;

        let (checked_variables, corresponding_lits): (Vec<_>, Vec<_>) =
            checked_variables.into_iter().unzip();

        quote! {
            let ( #( #bound_variables, )* ) = match #value {
                #pat if #( #checked_variables == #corresponding_lits && )* true => ( #( #bound_variables, )* ),

                _ => panic!("Matching failed"),
            };
        }
    }
}

impl Parse for BodyMatchCall {
    fn parse(input: ParseStream) -> syn::Result<BodyMatchCall> {
        Ok(BodyMatchCall {
            value: input.parse()?,
            _comma1: input.parse()?,
            pat: input.parse()?,
            _comma2: input.parse()?,
        })
    }
}

struct BodyMatchCall {
    value: Expr,
    _comma1: Token![,],
    pat: Pat,
    _comma2: Option<Token![,]>,
}

/// This type dives in a pattern, gathers information and alters it.
///
/// More precisely, it will:
///   - gather a list of all the variables that are bound by the pattern.
///   - replace each literal pattern with a binding of an internal ident and
///     keep track of which ident corresponds to what literal.
#[derive(Default)]
struct PatternVisitor {
    bound_variables: Vec<Ident>,
    checked_variables: Vec<(Ident, PatLit)>,
}

impl PatternVisitor {
    fn from_pattern(p: &mut Pat) -> PatternVisitor {
        let mut this = PatternVisitor::new();
        this.visit_pat_mut(p);

        this
    }

    fn new() -> PatternVisitor {
        PatternVisitor::default()
    }

    fn alter_lit_pattern(&mut self, pat: &mut Pat, pat_lit: PatLit) {
        let ident = self.add_checked_variable(pat_lit);
        let pat_ident = Self::mk_pat_ident(ident);
        *pat = Pat::Ident(pat_ident);
    }

    fn add_checked_variable(&mut self, pat_lit: PatLit) -> Ident {
        let ident = self.mk_checked_variables_internal_ident();
        self.checked_variables.push((ident.clone(), pat_lit));

        ident
    }

    fn mk_pat_ident(ident: Ident) -> PatIdent {
        PatIdent {
            attrs: Vec::new(),
            by_ref: None,
            mutability: None,
            ident,
            subpat: None,
        }
    }

    fn mk_checked_variables_internal_ident(&self) -> Ident {
        format_ident!("__restest_internal_{}", self.checked_variables.len())
    }
}

impl VisitMut for PatternVisitor {
    fn visit_pat_ident_mut(&mut self, i: &mut PatIdent) {
        self.bound_variables.push(i.ident.clone());
    }

    fn visit_pat_mut(&mut self, pat: &mut Pat) {
        match pat {
            Pat::Lit(pat_lit) => {
                let pat_lit = pat_lit.clone();
                self.alter_lit_pattern(pat, pat_lit)
            }

            _ => visit_mut::visit_pat_mut(self, pat),
        }
    }
}
