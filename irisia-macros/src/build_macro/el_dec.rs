use std::collections::{hash_map::Entry, HashMap};

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{braced, parse::ParseStream, Error, Expr, Ident, Result, Token, TypePath};

use crate::parse_incomplete::parse_maybe_incomplete_expr;

use super::Environment;

pub struct ElDecBuilder<'a> {
    env: &'a mut Environment,
    el_type: TypePath,
    props: HashMap<Ident, TokenStream /* as Expr */>,
    slot: TokenStream,
    on_create: Option<Expr>,
    as_child_data: Option<Expr>,
}

impl ElDecBuilder<'_> {
    pub fn parse(env: &mut Environment, input: ParseStream) -> Result<TokenStream> {
        let mut this = ElDecBuilder {
            env,
            el_type: input.parse()?,
            props: HashMap::new(),
            slot: quote! {()},
            on_create: None,
            as_child_data: None,
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
            "on_create" => Self::check_and_set_cmd(&mut self.on_create, &id, input),
            "data" => Self::check_and_set_cmd(&mut self.as_child_data, &id, input),
            other => Err(Error::new_spanned(
                &id,
                format!("unrecognized command `{other}`"),
            )),
        }
    }

    fn parse_prop(&mut self, input: ParseStream) -> Result<()> {
        let id: Ident = input.parse()?;
        input.parse::<Token![:]>()?;

        let value = self
            .env
            .borrow_wire(&parse_maybe_incomplete_expr(input, Token![,])); //input.parse()?;

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
            slot,
            on_create,
            as_child_data,
        } = self;

        let el_type = quote! {
            <#el_type as ::irisia::__macro_helper::ElementTypeHelper<_>>::Target
        };

        let props = props.iter().map(|(key, value)| {
            quote! {
                #key: ::irisia::element::FieldPlaceholder::initialized(#value),
            }
        });

        let as_child_data = match &as_child_data {
            Some(c) => c.clone().to_token_stream(),
            None => quote! {::std::default::Default::default()},
        };

        let env_vars = env.clone_env_wires();

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
            ::irisia::structure::single::<#el_type, _>(
                ::irisia::__macro_helper::ElementPropsAlias::<#el_type> {
                    #(#props)*
                    ..::std::default::Default::default()
                },
                #as_child_data,
                #slot,
                #on_create,
            )
        }
    }
}
