use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use std::collections::HashMap;
use syn::{
    token::{Brace, Paren},
    Attribute, Error, Fields, Ident, Member, Result, Token,
};

use self::{full_quality_paths::FullQualityPaths, read_fields::FieldAnalysis};
use super::attributes::get_attrs;
use crate::derive_style::attributes::{DeriveAttr, Segment};

pub mod full_quality_paths;
pub mod read_fields;

pub type FieldMap = HashMap<Member, FieldAnalysis>;

#[derive(Debug)]
pub struct VariantAnalysis {
    pub paths: FullQualityPaths,
    pub field_type: FieldType,
    pub fields: FieldMap,
    pub impl_default: bool,
    pub option: Option<Option<Ident>>,
}

#[derive(Debug, Clone, Copy)]
pub enum FieldType {
    Named,
    Unnamed,
    Unit,
}

impl VariantAnalysis {
    pub fn analyze_fields(attrs: &[Attribute], raw_fields: &Fields) -> Result<VariantAnalysis> {
        let mut paths = Vec::new();
        let mut fields = HashMap::new();
        let mut impl_default = false;
        let mut option = None;
        let field_type = match raw_fields {
            Fields::Named(_) => FieldType::Named,
            Fields::Unnamed(_) => FieldType::Unnamed,
            Fields::Unit => FieldType::Unit,
        };

        let mut auto_from: Vec<Segment> = Vec::new();
        for f in FieldAnalysis::analyze(raw_fields) {
            let f = f?;
            if !f.skip_auto_from {
                auto_from.push(Segment::Member(f.tag.clone()));
            }
            fields.insert(f.tag.clone(), f);
        }
        let mut auto_from = Some(auto_from);
        let error = |msg| Err(Error::new(Span::call_site(), msg));

        for attr in get_attrs(&attrs)? {
            match attr {
                DeriveAttr::From {
                    instruction: Some(field_paths),
                } => paths.extend(field_paths),
                DeriveAttr::From { instruction: None } => match auto_from.take() {
                    Some(af) => paths.push(af),
                    None => return error("auto from attribute has declared"),
                },
                DeriveAttr::ImplDefault => {
                    if impl_default {
                        return error("`impl_default` attribute has declared");
                    }
                    impl_default = true;
                }
                DeriveAttr::Option { rename, set_true } => {
                    if option.is_some() {
                        return error("`option` attribute has declared");
                    }

                    if set_true {
                        return error("`set_true` is not allowed at variant level option");
                    }

                    option = Some(rename);
                }
                _ => {
                    return Err(Error::new(
                        Span::call_site(),
                        format!("attribute `{}` is not allowed here", attr.attr_name()),
                    ))
                }
            }
        }

        Ok(VariantAnalysis {
            paths: FullQualityPaths::complete(paths, &fields)?,
            field_type,
            fields,
            impl_default,
            option,
        })
    }
}

impl FieldType {
    pub fn surround<'a, I, T>(&self, iter: I) -> TokenStream
    where
        I: Iterator<Item = (&'a Member, T)>,
        T: ToTokens,
    {
        let mut tokens = TokenStream::new();

        let f = |tokens: &mut _| {
            for (tag, value) in iter {
                match (tag, self) {
                    (Member::Named(id), Self::Named) => {
                        id.to_tokens(tokens);
                        <Token![:]>::default().to_tokens(tokens);
                    }
                    (Member::Unnamed(_), Self::Unnamed) => {}
                    _ => panic!("member and field type doesn't match: {tag:?} {self:?}"),
                }

                value.to_tokens(tokens);
                <Token![,]>::default().to_tokens(tokens);
            }
        };

        match self {
            FieldType::Named => Brace::default().surround(&mut tokens, f),
            FieldType::Unnamed => Paren::default().surround(&mut tokens, f),
            FieldType::Unit => {}
        }
        tokens
    }
}
