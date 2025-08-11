use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    braced, parse::ParseStream, parse_quote, spanned::Spanned, Error, Expr, ExprLet, Ident, Pat,
    Result, Token,
};

use super::{clone_env_raw, el_dec::ElDecBuilder, pat_bind::PatBinds, Environment};

mod kw {
    use syn::custom_keyword;
    custom_keyword!(input);
}

const MAX_CHAIN_TUPLE_LENGTH: usize = 25;

impl Environment {
    pub fn parse_node(&mut self, input: ParseStream) -> Result<TokenStream> {
        if input.peek(Token![match]) {
            self.parse_match(input)
        } else if input.peek(Token![if]) {
            self.parse_if(input)
        } else if input.peek(Token![for]) {
            self.parse_for(input)
        } else if input.peek(Token![..]) {
            self.parse_extern(input)
        } else {
            ElDecBuilder::parse(self, input)
        }
    }

    pub fn parse_statements(&mut self, input: ParseStream) -> Result<TokenStream> {
        let mut nodes = Vec::new();
        while !input.is_empty() {
            if input.peek(kw::input) {
                nodes.push(self.parse_input(input)?);
                break;
            }

            if input.peek(Token![let]) {
                nodes.push(self.parse_let(input)?);
                break;
            }

            nodes.push(self.parse_node(input)?);
        }

        let mut tokens;
        if !nodes.is_empty() {
            tokens = nodes.remove(0);
            for chunk in nodes.chunks(MAX_CHAIN_TUPLE_LENGTH - 1) {
                tokens = quote! {
                    (#tokens, #(#chunk,)*)
                };
            }
        } else {
            tokens = quote! {()};
        }

        Ok(tokens)
    }

    fn parse_input(&mut self, input: ParseStream) -> Result<TokenStream> {
        input.parse::<kw::input>()?;
        let mut count = 0;

        let result = (|| {
            while !input.peek(Token![;]) {
                let ident: Ident = input.parse()?;
                if self.accessable.contains_key(&ident) {
                    return Err(Error::new_spanned(
                        &ident,
                        format!("variable `{ident}` is already captured or newly defined"),
                    ));
                }

                self.push_env(ident);
                count += 1;
                if !input.peek(Token![;]) {
                    input.parse::<Token![,]>()?;
                }
            }

            input.parse::<Token![;]>()?;
            let body = self.parse_statements(input)?;
            let new_envs = clone_env_raw(self.vars[self.vars.len() - count..].iter());
            Ok(quote! {{
                #new_envs
                #body
            }})
        })();

        self.pop_env(count);
        result
    }

    fn parse_let(&mut self, input: ParseStream) -> Result<TokenStream> {
        input.parse::<Token![let]>()?;
        let pat = PatBinds::new(Pat::parse_multi_with_leading_vert(input)?, None);
        input.parse::<Token![=]>()?;
        let expr: Expr = input.parse()?;
        input.parse::<Token![;]>()?;

        let rest = self.bind_env(&pat, |env| env.parse_statements(input))?;
        let temp_var = format_ident!("__irisia_let_binds", span = expr.span());
        let binds = pat.bind_var_from_wire(&temp_var, &pat.pattern);
        Ok(quote! {{
            let #temp_var = #expr;
            #binds
            #rest
        }})
    }

    pub fn parse_block(&mut self, input: ParseStream) -> Result<TokenStream> {
        let content;
        braced!(content in input);
        self.parse_statements(&content)
    }

    fn parse_if(&mut self, input: ParseStream) -> Result<TokenStream> {
        struct Branch {
            pat: Option<PatBinds>,
            expr: Expr,
            body: TokenStream,
        }

        let mut branches: Vec<Branch> = Vec::new();
        let mut output = loop {
            input.parse::<Token![if]>()?;
            match input.call(Expr::parse_without_eager_brace)? {
                Expr::Let(ExprLet { pat, expr, .. }) => {
                    let pat_binds = PatBinds::new(*pat, None);
                    let body = self.bind_env(&pat_binds, |this| this.parse_block(input))?;
                    branches.push(Branch {
                        pat: Some(pat_binds),
                        expr: *expr,
                        body,
                    });
                }
                other => branches.push(Branch {
                    pat: None,
                    expr: other,
                    body: self.parse_block(input)?,
                }),
            }

            if !input.peek(Token![else]) {
                break quote! {()};
            }
            input.parse::<Token![else]>()?;

            if input.peek(Token![if]) {
                continue;
            }

            break self.parse_block(input)?;
        };

        for Branch { pat, expr, body } in branches.into_iter().rev() {
            output = match pat {
                Some(pat_binds) => self.pat_match_to_tokens(&expr, &pat_binds, body, output),
                None => self.cond_to_tokens(&expr, body, output),
            }
        }

        Ok(output)
    }

    fn parse_match(&mut self, input: ParseStream) -> Result<TokenStream> {
        input.parse::<Token![match]>()?;
        let cond_wire = Expr::parse_without_eager_brace(input)?;

        let content;
        braced!(content in input);

        let mut arms: Vec<(PatBinds, TokenStream)> = Vec::new();
        while !content.is_empty() {
            let pat = content.call(Pat::parse_multi_with_leading_vert)?;
            let if_guard: Option<Expr> = if content.peek(Token![if]) {
                input.parse::<Token![if]>()?;
                Some(input.parse()?)
            } else {
                None
            };
            let pat_binds = PatBinds::new(pat, if_guard);

            content.parse::<Token![=>]>()?;
            let body = self.bind_env(&pat_binds, |this| this.parse_node(input))?;

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }

            arms.push((pat_binds, body));
        }

        let mut arms_tokens = quote! {()};
        for (pat_binds, body) in arms.into_iter().rev() {
            arms_tokens = self.pat_match_to_tokens(
                &parse_quote! {
                    ::irisia::data_flow::Readable::read(&__irisia_cond)
                },
                &pat_binds,
                body,
                arms_tokens,
            );
        }

        Ok(quote! {
            {
                let __irisia_cond = #cond_wire;
                #arms_tokens
            }
        })
    }

    fn parse_for(&mut self, input: ParseStream) -> Result<TokenStream> {
        input.parse::<Token![for]>()?;
        let pat = PatBinds::new(input.call(Pat::parse_multi_with_leading_vert)?, None);
        input.parse::<Token![in]>()?;
        let iter = input.call(Expr::parse_without_eager_brace)?;

        let body = self.bind_env(&pat, |this| this.parse_block(input))?;

        Ok(self.repeat_to_tokens(&pat, &iter, body))
    }

    fn parse_extern(&mut self, input: ParseStream) -> Result<TokenStream> {
        input.parse::<Token![..]>()?;
        let expr: Expr = input.parse()?;
        Ok(self.borrow_wire(&expr))
    }
}
