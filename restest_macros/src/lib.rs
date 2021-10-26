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

    fn mk_checked_variables_internal_ident(&self) -> Ident {
        format_ident!("__restest_internal_{}", self.checked_variables.len())
    }
}

impl VisitMut for PatternVisitor {
    fn visit_pat_ident_mut(&mut self, i: &mut PatIdent) {
        self.bound_variables.push(i.ident.clone());
    }

    fn visit_pat_mut(&mut self, i: &mut Pat) {
        match i {
            Pat::Lit(lit) => {
                let ident = self.mk_checked_variables_internal_ident();
                self.checked_variables.push((ident.clone(), lit.clone()));

                *i = Pat::Ident(PatIdent {
                    attrs: Vec::new(),
                    // TODO
                    by_ref: None,
                    // TODO
                    mutability: None,
                    ident,
                    subpat: None,
                });
            }

            _ => visit_mut::visit_pat_mut(self, i),
        }
    }
}
