use syn::{
    Attribute, Error, Expr, Ident, Lit, Member, Meta, MetaList, MetaNameValue, NestedMeta, Result,
};

#[derive(Debug)]
pub enum DeriveAttr {
    Skip,
    From {
        expr: Option<Vec<Vec<Member>>>,
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
        if !attr.path.is_ident("cream") {
            return Ok(vec![]);
        }

        let meta = match attr.parse_meta()? {
            Meta::List(MetaList { path, nested, .. }) if path.is_ident("cream") => nested,
            other => {
                return Err(Error::new_spanned(other, "expected list"));
            }
        };

        let mut attrs = Vec::new();

        for meta2 in meta {
            let meta2 = match meta2 {
                NestedMeta::Meta(m) => m,
                NestedMeta::Lit(_) => continue,
            };

            let name = match meta2.path().get_ident() {
                Some(n) => n.to_string(),
                None => {
                    return Err(Error::new_spanned(
                        meta2,
                        "attribute path is unsupported".to_string(),
                    ))
                }
            };

            let a = match &*name {
                "impl_default" => Self::ImplDefault,
                "skip" => Self::Skip,
                "from" => Self::From {
                    expr: super::parse_paths::parse_paths(meta2)?,
                },
                "default" => parse_default(meta2)?,
                "option" => parse_option(meta2)?,
                other => {
                    return Err(Error::new_spanned(
                        meta2,
                        format!("unknown attribute `{other}`"),
                    ))
                }
            };

            attrs.push(a)
        }

        Ok(attrs)
    }
}

fn parse_default(meta: Meta) -> Result<DeriveAttr> {
    let lit = match meta {
        Meta::Path(_) => return Ok(DeriveAttr::Default { expr: None }),
        Meta::List(list) => return Err(Error::new_spanned(list, "unexpected meta-list")),
        Meta::NameValue(MetaNameValue { lit, .. }) => lit,
    };

    match lit {
        Lit::Str(s) => {
            syn::parse_str(&s.value()).map(|expr| DeriveAttr::Default { expr: Some(expr) })
        }
        _ => Err(Error::new_spanned(lit, "expected string literal")),
    }
}

fn parse_option(meta: Meta) -> Result<DeriveAttr> {
    match meta {
        Meta::NameValue(MetaNameValue {
            lit: Lit::Str(s), ..
        }) => syn::parse_str::<Ident>(&s.value()).map(|x| DeriveAttr::Option {
            rename: Some(x),
            set_true: false,
        }),

        Meta::List(MetaList { nested, .. }) => {
            let mut rename = None;
            let mut set_true = false;
            for nested_meta in nested.into_iter() {
                match nested_meta {
                    NestedMeta::Meta(Meta::Path(p)) if p.is_ident("set_true") => {
                        set_true = true;
                    }

                    NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                        path,
                        lit: Lit::Str(s),
                        ..
                    })) if path.is_ident("rename") => {
                        rename = Some(syn::parse_str::<Ident>(&s.value())?);
                    }

                    _ => continue,
                }
            }

            Ok(DeriveAttr::Option { rename, set_true })
        }

        _ => Ok(DeriveAttr::Option {
            rename: None,
            set_true: false,
        }),
    }
}
