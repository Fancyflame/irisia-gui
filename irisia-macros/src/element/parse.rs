use crate::{element::ElementCodegen, expr::state_block::parse_stmts, expr::StateExpr};
use std::collections::{hash_map::Entry, HashMap};
use syn::{braced, parse::Parse, Error, Expr, Ident, Token, Type};

pub struct ElementStmt {
    pub element: Type,
    pub props: HashMap<Ident, Expr>,
    pub style: Option<Expr>,
    pub oncreate: Option<Expr>,
    pub children: Vec<StateExpr<ElementCodegen>>,
}

impl Parse for ElementStmt {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let element: Type = input.parse()?;
        let mut props: HashMap<Ident, Expr> = HashMap::new();
        let mut style: Option<Expr> = None;
        let mut oncreate: Option<Expr> = None;

        let content;
        braced!(content in input);

        while !content.is_empty() {
            if content.peek(Ident) && content.peek2(Token![:]) {
                // parse props-value pair
                let ident: Ident = content.parse()?;
                match props.entry(ident) {
                    Entry::Occupied(occ) => {
                        return Err(Error::new(
                            occ.key().span(),
                            format!("duplicated property `{}`", occ.key()),
                        ))
                    }
                    Entry::Vacant(vac) => {
                        vac.insert({
                            content.parse::<Token![:]>()?;
                            content.parse()?
                        });
                    }
                }
            } else if content.peek(Token![+]) {
                // parse command
                content.parse::<Token![+]>()?;
                let cmd: Ident = content.parse()?;
                content.parse::<Token![:]>()?;

                let is_ok = match &*cmd.to_string() {
                    "style" => style.replace(content.parse()?).is_none(),
                    "oncreate" => oncreate.replace(content.parse()?).is_none(),
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
                // parse child nodes
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
            style,
            oncreate,
            children,
        })
    }
}
