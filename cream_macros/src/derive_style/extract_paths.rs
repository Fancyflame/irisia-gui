use std::collections::HashMap;

use proc_macro2::Span;
use syn::{
    spanned::Spanned, Attribute, Error, ExprPath, Fields, FieldsNamed, FieldsUnnamed, Ident, Index,
    Member, Result, Type,
};

use super::attr_parse::DeriveAttr;

#[derive(Debug)]
pub struct ExtractResult {
    pub paths: Vec<Vec<Member>>,
    pub metadatas: HashMap<Member, FieldMetadata>,
}

#[derive(Debug)]
pub struct FieldMetadata {
    pub default: Option<Option<ExprPath>>,
    pub ty: Type,
    pub option: Option<Ident>,
    pub option_set_true: bool,
}

pub fn analyze_fields(attrs: Vec<Attribute>, fields: &Fields) -> Result<ExtractResult> {
    let fields_and_attrs: Vec<(Member, Vec<DeriveAttr>, Type)> = fields_and_attrs(&fields)?;

    let paths: Vec<Vec<Member>> = {
        let mut vec = Vec::new();
        for attr in get_attrs(&attrs)? {
            match attr {
                DeriveAttr::From { expr: Some(p) } => vec.extend(p),
                DeriveAttr::From { expr: None } => vec.push(auto_from(&fields_and_attrs)),
                _ => continue,
            }
        }
        vec
    };

    let metadatas = get_metadata(&fields_and_attrs)?;

    Ok(ExtractResult { paths, metadatas })
}

fn fields_and_attrs(fields: &Fields) -> Result<Vec<(Member, Vec<DeriveAttr>, Type)>> {
    match &fields {
        Fields::Named(FieldsNamed { named: fields, .. })
        | Fields::Unnamed(FieldsUnnamed {
            unnamed: fields, ..
        }) => {
            let mut vec = Vec::new();
            for (index, field) in fields.iter().enumerate() {
                let member = match &field.ident {
                    Some(ident) => Member::Named(ident.clone()),
                    None => Member::Unnamed(Index {
                        index: index as _,
                        span: field.span(),
                    }),
                };

                vec.push((member, get_attrs(&field.attrs)?, field.ty.clone()))
            }
            Ok(vec)
        }
        Fields::Unit => Ok(Vec::new()),
    }
}

fn auto_from(faa: &[(Member, Vec<DeriveAttr>, Type)]) -> Vec<Member> {
    let mut path = Vec::new();

    for (member, attrs, _) in faa.into_iter() {
        if attrs
            .iter()
            .find(|p| match p {
                DeriveAttr::Skip => true,
                _ => false,
            })
            .is_some()
        {
            continue;
        }

        path.push(member.clone());
    }

    path
}

fn get_metadata(faa: &[(Member, Vec<DeriveAttr>, Type)]) -> Result<HashMap<Member, FieldMetadata>> {
    let mut map = HashMap::new();

    for (member, attrs, ty) in faa {
        let mut default = None;
        let mut option = None;
        let mut option_set_true = false;

        for attr in attrs {
            match attr {
                DeriveAttr::Default { with } => {
                    if default.replace(with.clone()).is_some() {
                        return Err(Error::new_spanned(member, "duplicated default declaration"));
                    }
                }

                DeriveAttr::Option { rename, set_true } => {
                    if option.is_some() {
                        return Err(Error::new(
                            Span::call_site(),
                            "duplicated option declaration",
                        ));
                    }

                    option_set_true = *set_true;

                    match (member, rename) {
                        (Member::Named(id), None) => option = Some(id.clone()),
                        (Member::Unnamed(index), None) => {
                            return Err(Error::new_spanned(index, "option name is required"))
                        }
                        (_, name @ Some(_)) => option = name.clone(),
                    }
                }

                _ => continue,
            }
        }
        map.insert(
            member.clone(),
            FieldMetadata {
                ty: ty.clone(),
                option,
                default,
                option_set_true,
            },
        );
    }
    Ok(map)
}

pub fn get_attrs(attrs: &[Attribute]) -> Result<Vec<DeriveAttr>> {
    let mut output = Vec::new();
    for attr in attrs {
        output.extend(DeriveAttr::parse_attr(attr)?);
    }
    Ok(output)
}
