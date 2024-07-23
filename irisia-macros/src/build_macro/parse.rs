use std::collections::{hash_map::Entry, HashMap};

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    braced, custom_keyword, parse::ParseStream, parse_quote, token::Brace, Error, Expr, ExprLet,
    Ident, Pat, Result, Token, TypePath,
};

use crate::parse_incomplete::parse_maybe_incomplete_expr;

use super::{env_to_tokens_raw, kw, pat_bind::PatBinds, Environment};

const MAX_CHAIN_TUPLE_LENGTH: usize = 25;

custom_keyword!(key);

impl Environment {
    pub fn parse_node(&mut self, input: ParseStream) -> Result<TokenStream> {
        if input.peek(Token![match]) {
            self.parse_match(input)
        } else if input.peek(Token![if]) {
            self.parse_if(input)
        } else if input.peek(Token![for]) {
            self.parse_for(input)
        } else if input.peek(Brace) {
            self.parse_extern(input)
        } else {
            ElDecBuilder::parse(self, input)
        }
    }

    pub fn parse_statements(&mut self, input: ParseStream) -> Result<TokenStream> {
        if input.peek(kw::input) {
            return self.parse_input(input);
        }

        let mut nodes = Vec::new();
        while !input.is_empty() {
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
                if self.vars.contains(&ident) {
                    return Err(Error::new_spanned(
                        &ident,
                        format!("variable `{ident}` is already captured or newly defined"),
                    ));
                }

                self.vars.push(ident);
                count += 1;
                if !input.peek(Token![;]) {
                    input.parse::<Token![,]>()?;
                }
            }

            input.parse::<Token![;]>()?;
            let body = self.parse_statements(input)?;
            let env = env_to_tokens_raw(&self.vars[self.vars.len() - count..], false);
            Ok(quote! {{
                #env
                #body
            }})
        })();

        self.pop_env(count);
        result
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
                None => self.cond_to_tokens(self.create_wire(&expr), body, output),
            }
        }

        Ok(output)
    }

    fn parse_match(&mut self, input: ParseStream) -> Result<TokenStream> {
        input.parse::<Token![match]>()?;
        let cond_wire = self.create_wire(&input.parse()?);

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
        let iter: Expr = input.parse()?;

        input.parse::<Token![,]>()?;
        input.parse::<kw::key>()?;
        input.parse::<Token![=]>()?;
        let key_expr = input.call(Expr::parse_without_eager_brace)?;

        let body = self.bind_env(&pat, |this| this.parse_block(input))?;

        Ok(self.repeat_to_tokens(&iter, &key_expr, &pat, body))
    }

    fn parse_extern(&mut self, input: ParseStream) -> Result<TokenStream> {
        let content;
        braced!(content in input);

        // we don't care about the content, maybe it is an incomplete expression
        let tokens = content.parse::<TokenStream>()?;

        let env = self.env_to_tokens();
        Ok(quote! {
            {
                #env
                #tokens
            }
        })
    }
}

pub struct ElDecBuilder<'a> {
    env: &'a mut Environment,
    el_type: TypePath,
    props: HashMap<Ident, Expr>,
    styles: Option<Expr>,
    slot: TokenStream,
    on_create: Option<Expr>,
}

impl ElDecBuilder<'_> {
    fn parse(env: &mut Environment, input: ParseStream) -> Result<TokenStream> {
        let mut this = ElDecBuilder {
            env,
            el_type: input.parse()?,
            props: HashMap::new(),
            styles: None,
            slot: quote! {()},
            on_create: None,
        };

        if input.peek(Token![;]) {
            input.parse::<Token![;]>()?;
            return Ok(this.to_tokens());
        }

        let content;
        braced!(content in input);
        loop {
            if content.peek(Token![@]) {
                this.parse_command(&content)?;
            } else if content.peek(Ident) && (content.peek2(Token![:]) || content.peek2(Token![<=]))
            {
                this.parse_prop(&content)?;
            } else {
                break;
            }

            if !content.is_empty() {
                content.parse::<Token![,]>()?;
            }
        }

        this.slot = this.env.parse_statements(&content)?;
        Ok(this.to_tokens())
    }

    fn check_and_set_cmd(cmd: &mut Option<Expr>, id: &Ident, input: ParseStream) -> Result<()> {
        if cmd.is_some() {
            Err(Error::new_spanned(
                id,
                format!("duplicated command declaration found `{id}`"),
            ))
        } else {
            *cmd = Some(parse_maybe_incomplete_expr(input, Token![,]));
            Ok(())
        }
    }

    fn parse_command(&mut self, input: ParseStream) -> Result<()> {
        input.parse::<Token![@]>()?;
        let id: Ident = input.parse()?;
        input.parse::<Token![:]>()?;

        match &*id.to_string() {
            "styles" => Self::check_and_set_cmd(&mut self.styles, &id, input),
            "on_create" => Self::check_and_set_cmd(&mut self.on_create, &id, input),
            other => Err(Error::new_spanned(
                &id,
                format!("unrecognized command `{other}`"),
            )),
        }
    }

    fn parse_prop(&mut self, input: ParseStream) -> Result<()> {
        let id: Ident = input.parse()?;

        let assign_mode = if input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            false
        } else {
            input.parse::<Token![<=]>()?;
            true
        };

        let mut value: Expr = parse_maybe_incomplete_expr(input, Token![,]); //input.parse()?;

        if !assign_mode {
            let raw = self.env.create_wire(&value);
            value = syn::parse2(raw.clone()).unwrap_or(Expr::Verbatim(raw));
        }

        match self.props.entry(id) {
            Entry::Occupied(occ) => Err(Error::new_spanned(
                &occ.key(),
                format!("property `{}` has been declared", occ.key()),
            )),
            Entry::Vacant(vac) => {
                vac.insert(value);
                Ok(())
            }
        }
    }

    fn to_tokens(self) -> TokenStream {
        let Self {
            env,
            el_type,
            props,
            styles,
            slot,
            on_create,
        } = self;

        // TODO: 等rust-analyzer把bug修了才能取消注释<https://github.com/rust-lang/rust-analyzer/issues/17651>
        /*let el_type = quote! {
            <#el_type as ::irisia::element::macro_helper::ElementTypeHelper<_>>::Target
        };*/

        let props = props.iter().map(|(key, value)| {
            quote! {
                #key: ::std::convert::From::from(#value),
            }
        });

        let env_vars = env.env_to_tokens();

        let styles = match styles {
            Some(styles) => quote! {{
                #env_vars
                #styles
            }},
            None => quote! {()},
        };

        let on_create = match on_create {
            Some(on_create) => quote! {{
                #env_vars
                #on_create
            }},
            None => quote! {
                |_| {}
            },
        };

        quote! {
            ::irisia::structure::single::<#el_type>(
                ::irisia::element::macro_helper::ElementPropsAlias::<#el_type> {
                    #(#props)*
                    ..::std::default::Default::default()
                },
                #styles,
                #slot,
                #on_create,
            )
        }
    }
}
