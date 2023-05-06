use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{parse::Parse, token::Brace, Expr, Result, Token};

use crate::expr::{state_block::StateBlock, Codegen, ConditionalApplicator, StateExpr, VisitUnit};

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
    default: Option<(Token![else], StateBlock<T>)>,
}

struct If<T: Codegen> {
    if_token: Token![if],
    cond: Expr,
    then: StateBlock<T>,
}

impl<T: Codegen> Parse for StateIf<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let leading_if = input.parse()?;

        let mut else_ifs = Vec::new();
        let mut default = None;
        while input.peek(Token![else]) {
            let else_token = input.parse()?;
            if input.peek2(Token![if]) {
                else_ifs.push((else_token, input.parse()?));
            } else {
                default = Some((else_token, input.parse()?));
                break;
            }
        }

        Ok(StateIf {
            leading_if,
            else_ifs,
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
    fn as_branch(&self, tokens: &mut TokenStream, applicator: &mut T::Ca) {
        let If {
            if_token,
            cond,
            then,
        } = self;

        if_token.to_tokens(tokens);
        cond.to_tokens(tokens);

        Brace::default().surround(tokens, |tokens| {
            applicator.apply(tokens, |tokens| {
                then.to_tokens(tokens);
            })
        });
    }
}

impl<T: Codegen> ToTokens for StateIf<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut ca = T::conditional_applicate(2 + self.else_ifs.len());

        self.leading_if.as_branch(tokens, &mut ca);

        for (else_token, if_expr) in self.else_ifs.iter() {
            else_token.to_tokens(tokens);
            if_expr.as_branch(tokens, &mut ca);
        }

        if let Some((else_token, block)) = &self.default {
            else_token.to_tokens(tokens);
            Brace::default().surround(tokens, |tokens| {
                ca.apply(tokens, |tokens| block.to_tokens(tokens))
            });
        } else {
            <Token![else]>::default().to_tokens(tokens);
            Brace::default().surround(tokens, |tokens| {
                ca.apply(tokens, |tokens| T::empty(tokens));
            })
        }
    }
}

impl<T: Codegen> VisitUnit<T> for StateIf<T> {
    fn visit_unit<'a, F>(&'a self, depth: usize, f: &mut F) -> Result<()>
    where
        F: FnMut(&'a StateExpr<T>, usize) -> Result<()>,
        T: 'a,
    {
        self.leading_if.then.visit_unit(depth, f)?;

        for x in &self.else_ifs {
            x.1.then.visit_unit(depth, f)?;
        }

        if let Some(def) = &self.default {
            def.1.visit_unit(depth, f)?;
        }

        Ok(())
    }

    fn visit_unit_mut<'a, F>(&'a mut self, depth: usize, f: &mut F) -> Result<()>
    where
        F: FnMut(&'a mut StateExpr<T>, usize) -> Result<()>,
        T: 'a,
    {
        self.leading_if.then.visit_unit_mut(depth, f)?;

        for x in &mut self.else_ifs {
            x.1.then.visit_unit_mut(depth, f)?;
        }

        if let Some(def) = &mut self.default {
            def.1.visit_unit_mut(depth, f)?;
        }

        Ok(())
    }
}
