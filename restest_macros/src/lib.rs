extern crate proc_macro;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::{Pair, Punctuated},
    token::{self, Brace, Bracket, Comma, Paren},
    visit_mut::{self, VisitMut},
    Arm, Expr, ExprIndex, ExprLit, ExprMacro, ExprMatch, ExprPath, ExprRange, ExprTuple, Ident,
    LitBool, Macro, MacroDelimiter, Pat, PatIdent, PatLit, PatSlice, PatTuple, PatWild, Path,
    RangeLimits, Token,
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

    fn expand_(self) -> TokenStream {
        let if_ = Token![if](Span::call_site());
        let true_ = Expr::from(ExprLit {
            attrs: Vec::new(),
            lit: LitBool::new(true, Span::call_site()).into(),
        });
        let guard = (if_, Box::new(true_));

        let match_expr = ModifiedMatchExpr::new_initial(self.value, self.pat, guard);

        match_expr
            .expand(Expr::Verbatim(quote! { todo!() }))
            .to_token_stream()
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

/// Allows to encode and expand a match expression that accepts slices patterns
/// over macros.
///
/// # Encoding
///
/// [`ModifiedMatchExpr`] can be created with
/// [`ModifiedMatchExpr::new_initial`]. It stores the expression that is being
/// matched on, initial pattern and the guard that ends the pattern.
///
/// # Expansion
///
/// Calling [`ModifiedMatchExpr::expand`] will return a nested `match`
/// expression. Each match expression will match on a new level of slicing.
///
/// Let's take the following macro as an example:
///
/// ```none
/// [                   // (1)
///     [a, 2, 3],
///     [b, 5, 6],
/// ]                   // (1)
/// ```
///
/// In the first iteration, we only alter the slice pattern delimited by `(1)`.
/// This will produce the following match expression:
///
/// ```none
/// match <expr> {
///     __restest__slice_0 => match (__restest__slice_0[..],) {
///         ([                    // (1)
///             [a, 2, 3],            // (2)
///             [b, 5, 6],            // (3)
///         ],) => { /* ... */ }, // (1)
///     }
/// }
/// ```
///
/// Next iteration must alter slice patterns `(2)` and `(3)` but must remove
/// slice pattern `(1)`, hence explaining why we treat differently recursion
/// compared to initialization. Altering `(1)` will result in infinitely doing
/// the same thing in a recursion function, which leads to a stack overflow.
///
/// As a result, the alteration of patterns `(2)` and `(3)` will yield to:
///
/// ```none
/// match <expr> {
///     __restest__slice_0 => match (__restest__slice_0[..],) {
///         ([
///             __restest__slice_0,
///             __restest__slice_1,
///         ],) => match (__restest__slice_0[..], __restest__slice_1[..],) {
///             (
///                 [a, 2, 3],
///                 [b, 5, 6],
///             ) => (a, b,),
///         }
///     }
/// }
/// ```
struct ModifiedMatchExpr {
    expr: Expr,
    pat: Pat,
    sub_match: MatchExprRecursion,
}

enum MatchExprRecursion {
    Recursive(Box<ModifiedMatchExpr>),
    Guarded((Token![if], Box<Expr>)),
}

impl ModifiedMatchExpr {
    fn new_initial(expr: Expr, pat: Pat, guard: (Token![if], Box<Expr>)) -> ModifiedMatchExpr {
        let mut replacer = SlicePatternReplacer::new();
        let pat = replacer.alter_initial_pattern(pat);

        Self::maybe_recurse(expr, pat, replacer, guard)
    }

    fn new_recursive(
        expr: Expr,
        pats: Vec<PatSlice>,
        guard: (Token![if], Box<Expr>),
    ) -> ModifiedMatchExpr {
        let mut replacer = SlicePatternReplacer::new();
        let pat = replacer.alter_sub_pattern(pats);

        Self::maybe_recurse(expr, pat, replacer, guard)
    }

    fn maybe_recurse(
        expr: Expr,
        pat: Pat,
        replacer: SlicePatternReplacer,
        guard: (token::If, Box<Expr>),
    ) -> ModifiedMatchExpr {
        let (extracted_values, corresponding_pattern) =
            replacer.extractable_tuple_and_corresponding_pattern();
        if extracted_values.elems.is_empty() {
            let sub_match = MatchExprRecursion::Guarded(guard);

            ModifiedMatchExpr {
                expr,
                pat,
                sub_match,
            }
        } else {
            let sub_expr = Expr::from(extracted_values);
            let sub_match =
                ModifiedMatchExpr::new_recursive(sub_expr, corresponding_pattern, guard);

            let sub_match = MatchExprRecursion::Recursive(Box::new(sub_match));

            ModifiedMatchExpr {
                expr,
                pat,
                sub_match,
            }
        }
    }

    fn expand(self, ending_expr: Expr) -> ExprMatch {
        let (guard, body) = match self.sub_match {
            MatchExprRecursion::Recursive(sub_match) => {
                (None, Expr::Match(sub_match.expand(ending_expr)))
            }
            MatchExprRecursion::Guarded(guard) => (Some(guard), ending_expr),
        };

        let expr = Box::new(self.expr);
        let arms = vec![Self::mk_arm(self.pat, guard, body), Self::catchall_arm()];

        ExprMatch {
            attrs: Vec::new(),
            match_token: Token![match](Span::call_site()),
            expr,
            brace_token: Brace {
                span: Span::call_site(),
            },
            arms,
        }
    }

    fn mk_arm(pat: Pat, guard: Option<(Token![if], Box<Expr>)>, body: Expr) -> Arm {
        let body = Box::new(body);
        Arm {
            attrs: Vec::new(),
            pat,
            guard,
            fat_arrow_token: Token![=>](Span::call_site()),
            body,
            comma: Some(Token![,](Span::call_site())),
        }
    }

    fn catchall_arm() -> Arm {
        Arm {
            attrs: Vec::new(),
            pat: Pat::Wild(PatWild {
                attrs: Vec::new(),
                underscore_token: Token![_](Span::mixed_site()),
            }),
            guard: None,
            fat_arrow_token: Token![=>](Span::mixed_site()),
            body: Box::new(Self::mk_panic_expr()),
            comma: Some(Token![,](Span::mixed_site())),
        }
    }

    fn mk_panic_expr() -> Expr {
        Expr::Macro(ExprMacro {
            attrs: Vec::new(),
            mac: Macro {
                path: Path::from(Ident::new("panic", Span::call_site())),
                bang_token: Token![!](Span::call_site()),
                delimiter: MacroDelimiter::Paren(Paren {
                    span: Span::call_site(),
                }),
                tokens: quote! { "Matching failed" },
            },
        })
    }
}

/// Alters slice pattern, stores it in memory and stores it internally.
///
/// We only alter outermost slice patterns. This process is repeated multiple
/// times.
struct SlicePatternReplacer {
    slices: Vec<(Ident, PatSlice)>,
}

impl SlicePatternReplacer {
    fn new() -> SlicePatternReplacer {
        SlicePatternReplacer { slices: Vec::new() }
    }

    fn alter_initial_pattern(&mut self, mut pat: Pat) -> Pat {
        self.visit_pat_mut(&mut pat);
        pat
    }

    fn alter_sub_pattern(&mut self, pat: Vec<PatSlice>) -> Pat {
        let elems = pat
            .into_iter()
            .map(|mut pat_slice| {
                self.visit_pat_slice_mut(&mut pat_slice);
                pat_slice
            })
            .collect();

        Self::mk_corresponding_pattern(elems).into()
    }

    fn extractable_tuple_and_corresponding_pattern(self) -> (ExprTuple, Vec<PatSlice>) {
        let (idents, pats): (Vec<_>, Vec<_>) = self.slices.into_iter().unzip();

        (Self::mk_extractable_tuple(idents), pats)
    }

    fn mk_extractable_tuple(idents: Vec<Ident>) -> ExprTuple {
        let elems: Punctuated<_, _> = idents
            .into_iter()
            .map(Self::expr_from_ident)
            .map(|expr| Pair::Punctuated(expr, Comma::default()))
            .collect();

        ExprTuple {
            attrs: Vec::new(),
            paren_token: Paren {
                span: Span::call_site(),
            },
            elems,
        }
    }

    fn mk_corresponding_pattern(pats: Vec<PatSlice>) -> PatTuple {
        let mut elems: Punctuated<_, _> = pats.into_iter().map(Self::pat_from_pat_slice).collect();
        elems.push_punct(Default::default());

        PatTuple {
            attrs: Vec::new(),
            paren_token: Paren {
                span: Span::call_site(),
            },
            elems,
        }
    }

    fn expr_from_ident(ident: Ident) -> Expr {
        // We need to match the Vec by value, so we must add [..] at the end of
        // each identifier.

        Expr::Index(ExprIndex {
            attrs: Vec::new(),
            expr: Box::new(Expr::Path(ExprPath {
                attrs: Vec::new(),
                qself: None,
                path: Path::from(ident),
            })),
            bracket_token: Bracket {
                span: Span::call_site(),
            },
            index: Box::new(Expr::Range(ExprRange {
                attrs: Vec::new(),
                from: None,
                limits: RangeLimits::HalfOpen(Token![..](Span::call_site())),
                to: None,
            })),
        })
    }

    fn pat_from_pat_slice(pat_slice: PatSlice) -> Pat {
        Pat::Slice(pat_slice)
    }

    fn add_slice_pattern(&mut self, pat: &mut Pat, slice: PatSlice) {
        let ident = self.mk_internal_slice_ident();
        self.slices.push((ident.clone(), slice));

        let pat_ident = PatIdent {
            attrs: Vec::new(),
            by_ref: None,
            mutability: None,
            ident,
            subpat: None,
        };

        *pat = Pat::Ident(pat_ident);
    }

    fn mk_internal_slice_ident(&self) -> Ident {
        format_ident!("__restest__array_{}", self.slices.len())
    }
}

impl VisitMut for SlicePatternReplacer {
    fn visit_pat_mut(&mut self, pat: &mut Pat) {
        match pat {
            Pat::Slice(slice) => {
                let slice = slice.clone();
                self.add_slice_pattern(pat, slice);
            }

            _ => visit_mut::visit_pat_mut(self, pat),
        }
    }
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

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn expand_2_base_case() {
        let call: BodyMatchCall = parse_quote! {
            foo,
            [a, b, c],
        };

        let left = call.expand_();

        let right = quote! {
            match foo {
                __restest__array_0 => match (__restest__array_0[..],) {
                    ([a, b, c],) => todo!(),
                    _ => panic!("Matching failed"),
                },
                _ => panic!("Matching failed"),
            }
        };

        assert_eq!(left.to_string(), right.to_string());
    }

    #[test]
    fn expand_2_with_recursion() {
        let call: BodyMatchCall = parse_quote! {
            foo,
            [[a], b, c],
        };

        let left = call.expand_();

        let right = quote! {
            match foo {
                __restest__array_0 => match (__restest__array_0[..],) {
                    ([__restest__array_0, b, c],) => match (__restest__array_0[..],) {
                        ([a],) => todo!(),
                        _ => panic!("Matching failed"),
                    },
                    _ => panic!("Matching failed"),
                },
                _ => panic!("Matching failed"),
            }
        };

        assert_eq!(left.to_string(), right.to_string());
    }

    #[test]
    fn expand_2_more_than_one() {
        let call: BodyMatchCall = parse_quote! {
            foo,
            ([foo], [bar]),
        };

        let left = call.expand_();

        let right = quote! {
            match foo {
                (__restest__array_0, __restest__array_1) => match (__restest__array_0[..], __restest__array_1[..],) {
                    ([foo], [bar],) => todo!(),
                    _ => panic!("Matching failed"),
                },
                _ => panic!("Matching failed"),
            }
        };

        assert_eq!(left.to_string(), right.to_string());
    }
}
