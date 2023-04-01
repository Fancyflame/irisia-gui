use syn::{
    braced,
    parse::{Parse, ParseStream},
    parse_quote, Error, Expr, Ident, LitStr, Result, Token, Type,
};

use crate::{element::stmt::ElementStmt, expr::state_block::parse_stmts};

impl Parse for ElementStmt {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let element: Type = input.parse()?;
        let mut props: Vec<(Ident, Expr)> = Vec::new();
        let mut rename: Option<Ident> = None;
        let mut style: Option<Expr> = None;
        let mut event_emitting_key: Option<Expr> = None;

        let content;
        braced!(content in input);

        while !content.is_empty() {
            if content.peek(Ident) && content.peek2(Token![:]) {
                // parse props-value pair
                props.push((content.parse()?, {
                    content.parse::<Token![:]>()?;
                    content.parse()?
                }));
            } else if content.peek(Token![+]) {
                content.parse::<Token![+]>()?;
                let cmd: Ident = content.parse()?;
                let is_ok = match &*cmd.to_string() {
                    "name" => rename.replace(call_rename(&content)?).is_none(),
                    "style" => style.replace(call_style(&content)?).is_none(),
                    "listen" => event_emitting_key.replace(call_listen(&content)?).is_none(),
                    other => {
                        return Err(Error::new(cmd.span(), format!("unknown command `{other}`")))
                    }
                };

                if !is_ok {
                    return Err(Error::new(
                        cmd.span(),
                        format!("duplicated command `{}`", cmd.to_string()),
                    ));
                }
            } else {
                break;
            }

            if !content.is_empty() {
                content.parse::<Token![,]>()?;
            }
        }

        let children = parse_stmts(&content)?;

        Ok(ElementStmt {
            element,
            props,
            rename,
            style: style.unwrap_or_else(|| parse_quote!(cream::style::NoStyle)),
            event_dispatcher: None,
            event_emitting_key,
            children,
        })
    }
}

fn call_rename(input: ParseStream) -> Result<Ident> {
    input.parse::<Token![:]>()?;
    let name: LitStr = input.parse()?;
    let mut name_ident = syn::parse_str::<Ident>(&name.value())?;
    name_ident.set_span(name.span());
    Ok(name_ident)
}

fn call_style(input: ParseStream) -> Result<Expr> {
    input.parse::<Token![:]>()?;
    input.parse()
}

fn call_listen(input: ParseStream) -> Result<Expr> {
    input.parse::<Token![:]>()?;
    input.parse()
}
