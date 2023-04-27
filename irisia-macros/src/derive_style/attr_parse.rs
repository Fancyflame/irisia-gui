use syn::{meta::ParseNestedMeta, token::Paren, Attribute, Expr, Ident, LitStr, Result, Token};

use super::parse_paths::Segment;

#[derive(Debug)]
pub enum DeriveAttr {
    Skip,
    From {
        expr: Option<Vec<Vec<Segment>>>,
    },
    Default {
        expr: Option<Expr>,
    },
    Option {
        rename: Option<Ident>,
        set_true: bool,
    },
    ImplDefault,
}

impl DeriveAttr {
    pub fn parse_attr(attr: &Attribute) -> Result<Vec<Self>> {
        if !attr.path().is_ident("irisia") {
            return Ok(vec![]);
        }

        let mut attrs = Vec::new();
        attr.parse_nested_meta(|meta| {
            attrs.push(parse_logic(meta)?);
            Ok(())
        })?;

        Ok(attrs)
    }
}

fn parse_logic(meta: ParseNestedMeta) -> Result<DeriveAttr> {
    let name = match meta.path.get_ident() {
        Some(name) => name.to_string(),
        None => return Err(meta.error("expected path")),
    };

    match &*name {
        "impl_default" => Ok(DeriveAttr::ImplDefault),
        "skip" => Ok(DeriveAttr::Skip),
        "from" => parse_from(meta),
        "default" => parse_default(meta),
        "option" => parse_option(meta),
        other => Err(meta.error(format!("unknown attribute `{other}`"))),
    }
}

fn parse_from(input: ParseNestedMeta) -> Result<DeriveAttr> {
    Ok(DeriveAttr::From {
        expr: if input.input.peek(Token![=]) {
            Some(super::parse_paths::parse_paths(input.value()?)?)
        } else {
            None
        },
    })
}

fn parse_default(input: ParseNestedMeta) -> Result<DeriveAttr> {
    if input.input.peek(Token![=]) {
        let litstr: LitStr = input.value()?.parse()?;
        syn::parse_str(&litstr.value()).map(|expr| DeriveAttr::Default { expr: Some(expr) })
    } else {
        Ok(DeriveAttr::Default { expr: None })
    }
}

fn parse_option(meta: ParseNestedMeta) -> Result<DeriveAttr> {
    let mut rename = None;
    let mut set_true = false;

    let mut exec_rename = |m: ParseNestedMeta| -> Result<()> {
        let value = m.value()?.parse::<LitStr>()?.value();
        rename = Some(syn::parse_str::<Ident>(&value)?);
        Ok(())
    };

    if meta.input.peek(Token![=]) {
        exec_rename(meta)?;
    } else if meta.input.peek(Paren) {
        meta.parse_nested_meta(|meta2| {
            let path2 = &meta2.path;
            if path2.is_ident("set_true") {
                set_true = true;
                Ok(())
            } else if path2.is_ident("rename") {
                exec_rename(meta2)?;
                Ok(())
            } else {
                match path2.get_ident() {
                    Some(i) => Err(meta2.error(format!("unknown method `{i}`"))),
                    None => Err(meta2.error("expected method name")),
                }
            }
        })?;
    }

    Ok(DeriveAttr::Option { rename, set_true })
}

impl DeriveAttr {
    pub fn attr_name(&self) -> &'static str {
        match self {
            DeriveAttr::Default { .. } => "default",
            DeriveAttr::From { .. } => "from",
            DeriveAttr::ImplDefault => "impl_default",
            DeriveAttr::Option { .. } => "option",
            DeriveAttr::Skip => "skip",
        }
    }
}
