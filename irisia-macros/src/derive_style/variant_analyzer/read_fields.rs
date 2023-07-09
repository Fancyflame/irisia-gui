use syn::{
    parse_quote, spanned::Spanned, Error, Expr, Fields, FieldsNamed, FieldsUnnamed, Ident, Index,
    Member, Result, Type,
};

use crate::derive_style::attributes::DeriveAttr;

use super::get_attrs;

#[derive(Debug, Clone)]
pub struct FieldAnalysis {
    pub tag: Member,
    pub default: Option<Expr>,
    pub ty: Type,
    pub option: Option<Ident>,
    pub option_set_true: bool,
    pub skip_auto_from: bool,
}

impl FieldAnalysis {
    pub fn analyze(fields: &Fields) -> impl Iterator<Item = Result<FieldAnalysis>> + '_ {
        read_fields(fields).map(analyze_one)
    }
}

fn analyze_one(field: Result<ReadField>) -> Result<FieldAnalysis> {
    let field = field?;
    let mut default = None;
    let mut option = None;
    let mut option_set_true = false;
    let mut skip_auto_from = false;

    let error_duplicated = |msg| Err(Error::new_spanned(&field.member, msg));

    for attr in field.attrs {
        match attr {
            DeriveAttr::Default { specified: expr } => {
                if default
                    .replace(
                        expr.unwrap_or_else(|| parse_quote!(::std::default::Default::default())),
                    )
                    .is_some()
                {
                    return error_duplicated("duplicated default declaration");
                }
            }

            DeriveAttr::Option { rename, set_true } => {
                if option.is_some() {
                    return error_duplicated("duplicated option declaration");
                }

                option_set_true = set_true;

                match (&field.member, rename) {
                    (Member::Named(id), None) => option = Some(id.clone()),
                    (Member::Unnamed(index), None) => {
                        return Err(Error::new_spanned(index, "option name is required"))
                    }
                    (_, name @ Some(_)) => option = name,
                }
            }

            DeriveAttr::Skip => {
                if skip_auto_from {
                    return error_duplicated("duplicated skip declaration");
                }
                skip_auto_from = true;
            }

            _ => continue,
        }
    }

    Ok(FieldAnalysis {
        tag: field.member,
        default,
        ty: field.ty,
        option,
        option_set_true,
        skip_auto_from,
    })
}

struct ReadField {
    attrs: Vec<DeriveAttr>,
    member: Member,
    ty: Type,
}

fn read_fields(fields: &Fields) -> impl Iterator<Item = Result<ReadField>> + '_ {
    let mut iter = match &fields {
        Fields::Named(FieldsNamed { named: fields, .. })
        | Fields::Unnamed(FieldsUnnamed {
            unnamed: fields, ..
        }) => Some(fields.iter().enumerate()),
        Fields::Unit => None,
    };

    std::iter::from_fn(move || {
        let iter = iter.as_mut()?;
        let (index, field) = iter.next()?;

        let member = match &field.ident {
            Some(ident) => Member::Named(ident.clone()),
            None => Member::Unnamed(Index {
                index: index as _,
                span: field.span(),
            }),
        };

        Some(get_attrs(&field.attrs).map(move |attrs| ReadField {
            attrs,
            member,
            ty: field.ty.clone(),
        }))
    })
}
