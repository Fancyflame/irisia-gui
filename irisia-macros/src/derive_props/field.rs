use attr_parser_fn::{
    find_attr,
    meta::{conflicts, key_str, key_value, path_only, ParseMetaExt},
    ParseArgs, ParseAttrTrait,
};

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, Field, Ident, Result, Type};

#[derive(Default)]
pub enum Defaulter {
    #[default]
    Required,
    Optioned,
    Default,
    DefaultWith(Expr),
}

pub struct FieldProps {
    pub ident: Ident,
    pub ty: Type,
    pub rename: Option<Ident>,
    pub defaulter: Defaulter,
}

impl FieldProps {
    pub fn parse(field: Field) -> Result<Self> {
        let Some(attr) = find_attr::only(&field.attrs, "props")? else {
            return Ok(FieldProps {
                ident: field.ident.unwrap(),
                ty: field.ty,
                rename: None,
                defaulter: Defaulter::Default,
            });
        };

        let (rename, defaulter) = ParseArgs::new()
            .meta((
                ("rename", key_str::<Ident>()).optional(),
                conflicts((
                    ("required", path_only()).value(Defaulter::Required),
                    ("default", path_only()).value(Defaulter::Default),
                    ("default", key_value::<Expr>()).map(Defaulter::DefaultWith),
                    ("optioned", path_only()).value(Defaulter::Optioned),
                ))
                .optional(),
            ))
            .parse_attr(attr)?
            .meta;

        Ok(FieldProps {
            ident: field.ident.unwrap(),
            ty: field.ty,
            rename,
            defaulter: defaulter.unwrap_or_default(),
        })
    }

    fn field_name(&self) -> &Ident {
        self.rename.as_ref().unwrap_or(&self.ident)
    }

    pub fn new_field(&self) -> TokenStream {
        let origin_type = &self.ty;
        let name = self.field_name();

        quote! {
            #name: ::irisia::element::FieldPlaceholder<#origin_type>,
        }
    }

    pub fn default_field(&self) -> TokenStream {
        let name = self.field_name();

        let method = match &self.defaulter {
            Defaulter::Required => {
                let name_str = name.to_string();
                quote! {
                    MustInit(#name_str)
                }
            }
            Defaulter::Default => quote! {
                OrDefault(::std::default::Default::default)
            },
            Defaulter::DefaultWith(with_expr) => quote! {
                OrDefault(|| {#with_expr})
            },
            Defaulter::Optioned => quote! {
                Optioned
            },
        };

        quote! {
            #name: ::irisia::element::FieldPlaceholder::#method,
        }
    }

    pub fn take_field(&self) -> TokenStream {
        let origin_name = &self.ident;
        let props_name = self.field_name();

        let take = match self.defaulter {
            Defaulter::Optioned => quote! { take_optioned },
            _ => quote! { take },
        };

        quote! {
            #origin_name: _props.#props_name.#take(),
        }
    }
}
