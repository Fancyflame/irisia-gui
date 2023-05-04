use syn::{
    braced,
    parse::{Parse, ParseStream},
    parse_quote, Error, Expr, Ident, Result, Token, Type,
};

use crate::{element::stmt::ElementStmt, expr::state_block::parse_stmts};

impl Parse for ElementStmt {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let element: Type = input.parse()?;
        let mut props: Vec<(Ident, Expr)> = Vec::new();
        let mut style: Option<Expr> = None;
        let mut oncreate: Option<Expr> = None;

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
                content.parse::<Token![:]>()?;

                let is_ok = match &*cmd.to_string() {
                    "style" => style.replace(call_style(&content)?).is_none(),
                    "oncreate" => oncreate.replace(call_oncreate(&content)?).is_none(),
                    other => {
                        return Err(Error::new(cmd.span(), format!("unknown command `{other}`")))
                    }
                };

                if !is_ok {
                    return Err(Error::new(
                        cmd.span(),
                        format!("duplicated command `{}`", cmd),
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
            style: style.unwrap_or_else(|| parse_quote!(irisia::style::NoStyle)),
            oncreate,
            children,
        })
    }
}

fn call_style(input: ParseStream) -> Result<Expr> {
    input.parse()
}

fn call_oncreate(input: ParseStream) -> Result<Expr> {
    input.parse()
}
