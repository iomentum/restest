extern crate proc_macro;

use std::{collections::VecDeque, iter};

use proc_macro2::Span;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::{Pair, Punctuated},
    token::{Brace, Comma, FatArrow, Paren},
    visit::Visit,
    visit_mut::{self, VisitMut},
    Arm, Expr, ExprLit, ExprMatch, ExprTuple, Ident, Lit, LitStr, Local, Pat, PatIdent, PatLit,
    PatSlice, PatTuple, PatWild, Stmt, Token,
};

#[proc_macro]
pub fn assert_body_matches(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as BodyMatchCall);

    proc_macro::TokenStream::from(input.expand().to_token_stream())
}

impl BodyMatchCall {
    fn expand(mut self) -> Stmt {
        // We need to do three things:
        //
        //   - extract the identifier that are brought in scope by the macro
        //     call,
        //
        //   - alter the pattern so that string literals allow to match String,
        //
        //   - transform the pattern in a nested match expression, with one
        //     level of nesting for each slice pattern.

        let let_token = Token![let](Span::call_site());
        let equal = Token![=](Span::call_site());
        let semi_token = Token![;](Span::call_site());

        let (bindings, return_expr) =
            BindingPatternsExtractor::new(&self.pat).expand_bindings_and_return_expr();
        let guard_condition = StringLiteralPatternModifier::new(&mut self.pat).expand_guard_expr();
        let match_expr =
            SlicePatternModifier::new(self.value, self.pat, guard_condition, return_expr.into())
                .expand();

        let pat = bindings.into();
        let match_expr = Box::new(match_expr.into());

        Stmt::Local(Local {
            attrs: Vec::new(),
            let_token,
            pat,
            init: Some((equal, match_expr)),
            semi_token,
        })
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

/// Allows to extract a list of all the identifiers that are brought in scope
/// by a given pattern.
///
/// This allows us to generate the final body of the innermost match pattern
/// that `assert_body_matches` will expand to.
///
/// # How
///
/// [`BindingPatternsExtractor`] implements [`Visit`], which allows to
/// recursively visit the entire AST. We use this to visit a given pattern
/// and ensure extract every binding pattern.
///
/// # Example
///
/// The following pattern:
///
/// ```none
/// Foo {
///     field,
///     inner: Bar {
///         value: 42,
///         other_value,
///     },
///     final_value,
/// }
/// ```
///
/// Brings the following identifiers in scope:
///   - `field`,
///   - `other_value`,
///   - `final_value`.
#[derive(Default)]
struct BindingPatternsExtractor<'pat> {
    bindings: Vec<&'pat Ident>,
}

impl<'pat> BindingPatternsExtractor<'pat> {
    fn new(pat: &'pat Pat) -> BindingPatternsExtractor<'pat> {
        let mut this = BindingPatternsExtractor::default();
        this.visit_pat(pat);
        this
    }

    fn expand_bindings_and_return_expr(self) -> (PatTuple, ExprTuple) {
        let paren_token = Paren {
            span: Span::call_site(),
        };

        let bindings = self.mk_bindings(paren_token);
        let return_expr = self.mk_return_expr(paren_token);

        (bindings, return_expr)
    }

    fn mk_bindings(&self, paren_token: Paren) -> PatTuple {
        let elems = self
            .bindings
            .iter()
            .copied()
            .cloned()
            .map(Self::mk_ident_pat)
            .map(|i| Pair::Punctuated(i, Comma::default()))
            .collect();

        PatTuple {
            attrs: Vec::new(),
            paren_token,
            elems,
        }
    }

    fn mk_return_expr(self, paren_token: Paren) -> ExprTuple {
        let elems = self
            .bindings
            .into_iter()
            .cloned()
            .map(Self::mk_ident_expr)
            .map(|i| Pair::Punctuated(i, Comma::default()))
            .collect::<Punctuated<_, _>>();

        ExprTuple {
            attrs: Vec::new(),
            paren_token,
            elems,
        }
    }

    fn mk_ident_expr(ident: Ident) -> Expr {
        Expr::Verbatim(quote! { #ident })
    }

    fn mk_ident_pat(ident: Ident) -> Pat {
        Pat::Ident(PatIdent {
            attrs: Vec::new(),
            by_ref: None,
            mutability: None,
            ident,
            subpat: None,
        })
    }
}

impl<'pat> Visit<'pat> for BindingPatternsExtractor<'pat> {
    fn visit_pat_ident(&mut self, i: &'pat PatIdent) {
        self.bindings.push(&i.ident);
    }
}

/// Allows to perform pattern matching over `String` using literals.
///
/// To do so, we need to alter the pattern and change every instance of string
/// literal pattern into a binding and check for equality in the final guard.
///
/// # How
///
/// [`StringLiteralPatternModifier`] implements [`VisitMut`], which allows us to
/// recursively visit and alter the AST. Here, we use this to visit and alter a
/// given pattern. Specifically, we change all the string literal pattern to be
/// bindings of a given identifier and keep track of which identifier
/// corresponds to which string literal.
///
/// # Example
///
/// The following pattern:
///
/// ```none
/// Foo {
///     field: "string literal 1",
///     inner: Bar {
///         other_field: "string literal 2",
///     },
///     final: "string literal 3",
/// }
/// ```
///
/// Will be transformed to:
///
/// ```none
/// Foo {
///     field: __restest__str_0,
///     inner: Bar {
///         other_field: __restest__str_1,
///     },
///     final: __restest__str_2,
/// }
/// ```
///
/// And will generate the following conditions:
///   - `__restest__str_0 == "string literal 1"`,
///   - `__restest__str_1 == "string literal 2"`,
///   - `__restest__str_2 == "string literal 3"`.
#[derive(Default)]
struct StringLiteralPatternModifier {
    conditions: Vec<(Ident, LitStr)>,
}

impl StringLiteralPatternModifier {
    fn new(pat: &mut Pat) -> StringLiteralPatternModifier {
        let mut this = StringLiteralPatternModifier::default();

        this.visit_pat_mut(pat);
        this
    }

    fn expand_guard_expr(self) -> Expr {
        let (names, values): (Vec<_>, Vec<_>) = self.conditions.into_iter().unzip();
        Expr::Verbatim(quote! {
            true #( && #names == #values )*
        })
    }

    fn add_literal_pattern(&mut self, lit: LitStr) -> Ident {
        let name = self.mk_ident();
        self.conditions.push((name.clone(), lit));
        name
    }

    fn alter_pattern(pat: &mut Pat, ident: Ident) {
        *pat = Pat::Ident(PatIdent {
            attrs: Vec::new(),
            by_ref: None,
            mutability: None,
            ident,
            subpat: None,
        })
    }

    fn mk_ident(&self) -> Ident {
        format_ident!("__restest__str_{}", self.conditions.len())
    }
}

impl VisitMut for StringLiteralPatternModifier {
    fn visit_pat_mut(&mut self, pat: &mut Pat) {
        match pat {
            Pat::Lit(PatLit { expr, .. }) => match expr.as_ref() {
                Expr::Lit(ExprLit {
                    lit: Lit::Str(lit), ..
                }) => {
                    let ident = self.add_literal_pattern(lit.clone());
                    Self::alter_pattern(pat, ident);
                }

                _ => visit_mut::visit_pat_mut(self, pat),
            },

            _ => visit_mut::visit_pat_mut(self, pat),
        }
    }
}

/// Allows to encode and expand a match expression that accepts slices patterns
/// for `Vec`.
///
/// # How
///
/// We use [`VisitMut`] to visit and alter the pattern. We transform every slice
/// pattern into a binding of a unique identifier, and match on its content in
/// an inner expression.
///
/// This results in multiple, nested match expressions, each of them matching
/// over exactly one slice pattern.
///
/// # Example
///
/// Given the following pattern:
///
/// ```none
/// [                   // (1)
///     [a, 2, 3],          // (2)
///     [b, 5, 6],          // (3)
/// ]                   // (1)
/// ```
///
/// In the first iteration, we only alter the slice pattern delimited by `(1)`.
/// This will produce the following match expression:
///
/// ```none
/// match <expr> {
///     __restest__slice_0 => match __restest__slice_0[..] {
///         [__restest__slice_1, __restest__slice_2] => { /* ... */ },
///     }
/// }
/// ```
///
/// Next iteration alters slice pattern `(2)` and leads us to:
///
/// ```none
/// match <expr> {
///     __restest__slice_0 => match __restest__slice_0[..] {
///         [__restest__slice_1, __restest__slice_2] => match __restest__slice_1[..] {
///             [a, 2, 3] => { /* ... */ },
///         }
///     }
/// }
/// ```
///
/// The last iteration alters slice pattern `(3)` and expands to:
///
/// ```none
/// match <expr> {
///     __restest__slice_0 => match __restest__slice_0[..] {
///         [__restest__slice_1, __restest__slice_2] => match __restest__slice_1[..] {
///             [a, 2, 3] => match __restest__slice_2[..] {
///                 [b, 5, 6] => { /* final expression */ }
///             },
///         }
///     }
/// }
/// ```
struct SlicePatternModifier {
    first_expr: Expr,
    first_pat: Pat,
    nested_matches: Vec<(Expr, Pat)>,
    final_guard_condition: Expr,
    return_expr: Expr,
}

impl SlicePatternModifier {
    fn new(
        matched_expr: Expr,
        pat: Pat,
        final_guard_condition: Expr,
        return_expr: Expr,
    ) -> SlicePatternModifier {
        let mut sub_slice_patterns = Vec::new();

        let mut replacer = SlicePatternReplacer::new();
        let pat = replacer.alter_initial_pattern(pat);

        let mut unaltered_slice_patterns = VecDeque::from_iter(replacer.extracted_slice_patterns());

        while let Some((ident, pat)) = unaltered_slice_patterns.pop_front() {
            let mut replacer = SlicePatternReplacer::new();
            let expr = Self::mk_match_expr(ident);
            let pat = replacer.alter_pat_slice(pat).into();

            sub_slice_patterns.push((expr, pat));
            unaltered_slice_patterns.extend(replacer.extracted_slice_patterns());
        }

        SlicePatternModifier {
            first_expr: matched_expr,
            first_pat: pat,
            nested_matches: sub_slice_patterns,
            final_guard_condition,
            return_expr,
        }
    }

    fn expand(self) -> ExprMatch {
        let mut nesting = iter::once((self.first_expr, self.first_pat))
            .chain(self.nested_matches)
            .rev();

        let (innermost_expr, innermost_pat) = nesting.next().unwrap();

        let guard = (
            <Token![if]>::default(),
            Box::new(self.final_guard_condition),
        );

        let arms = vec![
            Self::mk_arm(innermost_pat, Some(guard), self.return_expr),
            Self::catchall_arm(),
        ];

        let innermost_match = ExprMatch {
            attrs: Vec::new(),
            match_token: <Token![match]>::default(),
            expr: Box::new(innermost_expr),
            brace_token: Brace::default(),
            arms,
        };

        nesting.fold(innermost_match, Self::nest_match)
    }

    fn nest_match(inner: ExprMatch, (expr, pat): (Expr, Pat)) -> ExprMatch {
        let match_token = <Token![match]>::default();
        let expr = Box::new(expr);
        let brace_token = Brace::default();
        let arms = vec![Self::mk_arm(pat, None, inner.into()), Self::catchall_arm()];

        ExprMatch {
            attrs: Vec::new(),
            match_token,
            expr,
            brace_token,
            arms,
        }
    }

    fn mk_arm(pat: Pat, guard: Option<(Token![if], Box<Expr>)>, body: Expr) -> Arm {
        let body = Box::new(body);
        Arm {
            attrs: Vec::new(),
            pat,
            guard,
            fat_arrow_token: FatArrow::default(),
            body,
            comma: Some(Comma::default()),
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

    fn mk_match_expr(ident: Ident) -> Expr {
        Expr::Verbatim(quote! { #ident[..] })
    }

    fn mk_panic_expr() -> Expr {
        Expr::Verbatim(quote! { panic!("Matching failed")})
    }
}

/// Helper struct for [`SlicePatternReplacer`].
///
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

    fn alter_pat_slice(&mut self, mut pat: PatSlice) -> PatSlice {
        self.visit_pat_slice_mut(&mut pat);
        pat
    }

    fn extracted_slice_patterns(self) -> Vec<(Ident, PatSlice)> {
        self.slices
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

#[cfg(test)]
mod tests {
    use quote::ToTokens;
    use syn::parse_quote;

    use super::*;

    mod binding_patterns_extractor {
        use super::*;

        #[test]
        fn extraction_simple() {
            let pat = parse_quote! { foo };

            let return_expr = BindingPatternsExtractor::new(&pat)
                .expand_bindings_and_return_expr()
                .1;

            let left = return_expr.to_token_stream().to_string();
            let right = quote! { (foo,) }.to_string();

            assert_eq!(left, right);
        }

        #[test]
        fn extraction_in_subpattern() {
            let pat = parse_quote! { [foo, bar, .., baz ]};

            let return_expr = BindingPatternsExtractor::new(&pat)
                .expand_bindings_and_return_expr()
                .1;

            let left = return_expr.to_token_stream().to_string();
            let right = quote! { (foo, bar, baz,) }.to_string();

            assert_eq!(left, right);
        }

        #[test]
        fn handles_at_pattern() {
            let pat = parse_quote! { foo @ [] };

            let return_expr = BindingPatternsExtractor::new(&pat)
                .expand_bindings_and_return_expr()
                .1;

            let left = return_expr.to_token_stream().to_string();
            let right = quote! { (foo,) }.to_string();

            assert_eq!(left, right);
        }
    }

    mod string_literal_modifier {
        use super::*;

        #[test]
        fn simple_alteration() {
            let mut pat = parse_quote! { "foo" };

            let _ = StringLiteralPatternModifier::new(&mut pat);

            let left = pat.to_token_stream().to_string();
            let right = quote! {
                __restest__str_0
            }
            .to_string();

            assert_eq!(left, right);
        }

        #[test]
        fn simple_guard_condition() {
            let mut pat = parse_quote! { "foo" };

            let modifier = StringLiteralPatternModifier::new(&mut pat);

            let left = modifier.expand_guard_expr().to_token_stream().to_string();
            let right = quote! {
                true && __restest__str_0 == "foo"
            }
            .to_string();

            assert_eq!(left, right);
        }

        #[test]
        fn alteration_in_subpatterns() {
            let mut pat = parse_quote! {
                [
                    Foo { bar: "bar" },
                    ("42"),
                    [["hello"]],
                ]
            };

            let _ = StringLiteralPatternModifier::new(&mut pat);

            let left = pat.to_token_stream().to_string();
            let right = quote! {
                [
                    Foo { bar: __restest__str_0 },
                    (__restest__str_1),
                    [[__restest__str_2]],
                ]
            }
            .to_string();

            assert_eq!(left, right);
        }

        #[test]
        fn expansion_in_subpatterns() {
            let mut pat = parse_quote! {
                [
                    Foo { bar: "bar" },
                    ("42"),
                    [["hello"]],
                ]
            };

            let left = StringLiteralPatternModifier::new(&mut pat)
                .expand_guard_expr()
                .to_token_stream()
                .to_string();

            let right = quote! {
                true
                    && __restest__str_0 == "bar"
                    && __restest__str_1 == "42"
                    && __restest__str_2 == "hello"
            }
            .to_string();

            assert_eq!(left, right);
        }
    }

    #[test]
    fn expand_2_base_case() {
        let call: BodyMatchCall = parse_quote! {
            foo,
            [a, b, c],
        };

        let left = call.expand().to_token_stream().to_string();

        let right = quote! {
            let (a, b, c,) = match foo {
                __restest__array_0 => match __restest__array_0[..] {
                    [a, b, c] if true => (a, b, c,),
                    _ => panic!("Matching failed"),
                },
                _ => panic!("Matching failed"),
            };
        }
        .to_string();

        assert_eq!(left, right);
    }

    #[test]
    fn expand_2_with_recursion() {
        let call: BodyMatchCall = parse_quote! {
            foo,
            [[a], b, c],
        };

        let left = call.expand().to_token_stream().to_string();

        let right = quote! {
            let (a, b, c,) = match foo {
                __restest__array_0 => match __restest__array_0[..] {
                    [__restest__array_0, b, c] => match __restest__array_0[..] {
                        [a] if true => (a, b, c,),
                        _ => panic!("Matching failed"),
                    },
                    _ => panic!("Matching failed"),
                },
                _ => panic!("Matching failed"),
            };
        }
        .to_string();

        assert_eq!(left, right);
    }

    #[test]
    fn expand_2_more_than_one() {
        let call: BodyMatchCall = parse_quote! {
            foo,
            ([foo], [bar]),
        };

        let left = call.expand().to_token_stream().to_string();

        let right = quote! {
            let (foo, bar,) = match foo {
                (__restest__array_0, __restest__array_1) => match __restest__array_0[..] {
                    [foo] => match __restest__array_1[..] {
                        [bar] if true => (foo, bar,),
                        _ => panic!("Matching failed"),
                    },
                    _ => panic!("Matching failed"),
                },
                _ => panic!("Matching failed"),
            };
        }
        .to_string();

        assert_eq!(left, right);
    }
}
