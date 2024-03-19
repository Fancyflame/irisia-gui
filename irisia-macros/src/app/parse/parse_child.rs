use crate::expr::StmtTree;
use std::collections::{hash_map::Entry, HashMap};
use syn::{braced, parse::Parse, Error, Expr, Ident, Token, Type};

impl StmtTree for ElementStmt {
    type Command = ();
    type Stmt = Self;
    const MUST_IN_BLOCK: bool = false;
}

pub struct ElementStmt {
    pub element: Type,
    pub props: HashMap<Ident, Expr>,
    pub styles: Option<Expr>,
    pub oncreate: Option<Expr>,
    pub child: Box<Self>,
}

impl Parse for ElementStmt {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let element: Type = input.parse()?;
        let mut props: HashMap<Ident, Expr> = HashMap::new();
        let mut styles: Option<Expr> = None;
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
                    "style" => styles.replace(content.parse()?).is_none(),
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

        Ok(ElementStmt {
            element,
            props,
            styles,
            oncreate,
            child: content.parse()?,
        })
    }
}