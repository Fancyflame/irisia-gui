use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    braced,
    parse::{Nothing, Parse},
    punctuated::Punctuated,
    token::Brace,
    Expr, Pat, Token,
};

use crate::expr::{Codegen, StateExpr};

/*
    match expr1 {
        Pat1(x) if expr2 => Arm1(...),
        Pat2(y) => Arm2(Arm1(...)),
        Pat3(z) => Arm2(Arm2(...)),
    }
*/
pub struct StateMatch<T: Codegen> {
    match_token: Token![match],
    expr: Expr,
    brace_token: Brace,
    arms: Vec<Arm<T>>,
}

pub struct Arm<T: Codegen> {
    pat: Pat,
    guard: Option<(Token![if], Expr)>,
    fat_arrow_token: Token![=>],
    body: StateExpr<T>,
    comma: Option<Token![,]>,
}

impl<T: Codegen> StateMatch<T> {
    pub fn arms(&self) -> impl Iterator<Item = &StateExpr<T>> {
        self.arms.iter().map(|arm| &arm.body)
    }
}

impl<T: Codegen> Parse for StateMatch<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let match_token = input.parse()?;
        let expr = Expr::parse_without_eager_brace(input)?;

        let match_body;
        let brace_token = braced!(match_body in input);
        let arms = Punctuated::<_, Nothing>::parse_terminated_with(&match_body, |input| {
            Ok(Arm {
                pat: Pat::parse_multi_with_leading_vert(input)?,
                guard: {
                    if input.peek(Token![if]) {
                        Some((input.parse()?, input.parse()?))
                    } else {
                        None
                    }
                },
                fat_arrow_token: input.parse()?,
                body: {
                    let body: StateExpr<T> = input.parse()?;
                    if matches!(body, StateExpr::Raw(_)) && T::MUST_IN_BLOCK {
                        return Err(input.error(
                            "expression must be in block, consider move it into braces: `{expression}`.",
                        ));
                    }
                    body
                },
                comma: input.parse()?
            })
        })?
        .into_iter()
        .collect();

        Ok(StateMatch {
            match_token,
            expr,
            brace_token,
            arms,
        })
    }
}

impl<T: Codegen> ToTokens for StateMatch<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let StateMatch {
            match_token,
            expr,
            brace_token: _,
            arms,
        } = self;

        match_token.to_tokens(tokens);
        expr.to_tokens(tokens);
        self.brace_token.surround(tokens, |tokens| {
            let total = arms.len();
            for (index, arm) in arms.iter().enumerate() {
                let mut t = TokenStream::new();
                arm_to_tokens(&mut t, arm, index, total);
                *tokens = T::chain_applicate(&tokens, t);
            }
        });
    }
}

// pat if guard => expr,
fn arm_to_tokens<T: Codegen>(tokens: &mut TokenStream, arm: &Arm<T>, index: usize, total: usize) {
    let Arm {
        pat,
        guard,
        fat_arrow_token,
        body,
        comma,
    } = arm;

    pat.to_tokens(tokens);

    if let Some((if_token, expr)) = guard {
        if_token.to_tokens(tokens);
        expr.to_tokens(tokens);
    }

    fat_arrow_token.to_tokens(tokens);
    tokens.extend(T::conditional_applicate(body, index, total));

    match comma {
        Some(c) => c.to_tokens(tokens),
        None => <Token![,]>::default().to_tokens(tokens),
    }
}
