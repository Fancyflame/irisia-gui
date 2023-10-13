use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::Parse, token::Brace, Expr, Token};

use crate::expr::{state_block::StateBlock, Codegen};

/*
    // count 4
    if expr {
        Arm1(xxx) // skipped 0
    } else if expr2 {
        Arm2(Arm1(xxx)) // skipped 1
    } else if expr3 {
        Arm2(Arm2(Arm1(xxx))) // skipped 2
    } else {
        Arm2(Arm2(Arm2(xxx))) // skipped 3
    }
*/
pub struct StateIf<T: Codegen> {
    leading_if: If<T>,
    else_ifs: Vec<(Token![else], If<T>)>,
    default_else: Option<Token![else]>,
    default: StateBlock<T>,
}

struct If<T: Codegen> {
    if_token: Token![if],
    cond: Expr,
    then: StateBlock<T>,
}

impl<T: Codegen> StateIf<T> {
    pub fn arms(&self) -> impl Iterator<Item = &StateBlock<T>> {
        std::iter::once(&self.leading_if.then)
            .chain(self.else_ifs.iter().map(|(_, i)| &i.then))
            .chain(std::iter::once(&self.default))
    }
}

impl<T: Codegen> Parse for StateIf<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let leading_if = input.parse()?;

        let mut else_ifs = Vec::new();
        let mut default: Option<(Token![else], StateBlock<T>)> = None;

        while input.peek(Token![else]) {
            let else_token = input.parse()?;
            if input.peek2(Token![if]) {
                else_ifs.push((else_token, input.parse()?));
            } else {
                default = Some((else_token, input.parse()?));
                break;
            }
        }

        let (default_else, default) = match default {
            Some((et, df)) => (Some(et), df),
            None => Default::default(),
        };

        Ok(StateIf {
            leading_if,
            else_ifs,
            default_else,
            default,
        })
    }
}

impl<T: Codegen> Parse for If<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(If {
            if_token: input.parse()?,
            cond: Expr::parse_without_eager_brace(input)?,
            then: input.parse()?,
        })
    }
}

impl<T: Codegen> If<T> {
    fn as_branch(&self, tokens: &mut TokenStream, index: usize, total: usize) {
        let If {
            if_token,
            cond,
            then,
        } = self;

        let then = T::conditional_applicate(then, index, total);

        tokens.extend(quote! {
            #if_token #cond {
                #then
            }
        });
    }
}

impl<T: Codegen> ToTokens for StateIf<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut index = 0;

        // +2: leading if and trailing else
        let total = self.else_ifs.len() + 2;

        self.leading_if.as_branch(tokens, index, total);
        index += 1;

        for (else_token, if_expr) in self.else_ifs.iter() {
            else_token.to_tokens(tokens);
            if_expr.as_branch(tokens, index, total);
            index += 1;
        }

        match &self.default_else {
            Some(t) => t.to_tokens(tokens),
            None => <Token![else]>::default().to_tokens(tokens),
        }

        Brace::default().surround(tokens, |tokens| {
            tokens.extend(T::conditional_applicate(&self.default, total - 1, total))
        });
    }
}
