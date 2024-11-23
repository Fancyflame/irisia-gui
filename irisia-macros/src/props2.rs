use std::{iter::repeat, mem::replace};

use attr_parser_fn::{
    find_attr,
    meta::{conflicts, key_value, path_only, ParseMetaExt},
    ParseArgs, ParseAttrTrait,
};
use case::CaseExt;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote, Error, Expr, Field, Generics, Ident, ItemStruct, Result, Token, Type,
};

use crate::split_generics::SplittedGenerics;

pub struct CastProp {
    origin_generics: SplittedGenerics,
    item: ItemStruct,
    extra_infos: Vec<ExtraFieldInfo>,
}

impl Parse for CastProp {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut item_struct: ItemStruct = input.parse()?;
        let mut extra_infos: Vec<ExtraFieldInfo> = Vec::new();
        let origin_generics = SplittedGenerics::split_from_generics(item_struct.generics.clone());

        let type_of_uninited = quote! {irisia::element::deps::NeedInit};
        for field in item_struct.fields.iter_mut() {
            let behavior = parse_and_remove_props_attr(field)?;

            let type_of_inited = {
                let ty = &field.ty;
                quote! {irisia::hook::SimpleProvider<#ty>}
            };

            let extra_info = match behavior {
                Behavior::Required => {
                    let generic = format_insert_generic(&field)?;
                    item_struct.generics.params.push(parse_quote!(
                        #generic = #type_of_inited
                    ));

                    ExtraFieldInfo {
                        default_value: parse_quote!(#type_of_uninited),
                        origin_type: replace(&mut field.ty, parse_quote!(#generic)),
                        generic_type: Some(generic),
                    }
                }
                Behavior::Optional(default_optioned) => ExtraFieldInfo {
                    default_value: {
                        let default = default_optioned
                            .unwrap_or_else(|| parse_quote!(::std::default::Default::default()));
                        parse_quote!(
                            irisia::hook::SimpleProvider::Owned(#default)
                        )
                    },
                    origin_type: replace(&mut field.ty, parse_quote!(#type_of_inited)),
                    generic_type: None,
                },
            };
            extra_infos.push(extra_info);
        }

        Ok(Self {
            origin_generics,
            item: item_struct,
            extra_infos,
        })
    }
}

struct ExtraFieldInfo {
    default_value: Expr,
    origin_type: Type,
    generic_type: Option<Ident>,
}

impl CastProp {
    fn impl_default(&self) -> TokenStream {
        let SplittedGenerics {
            lifetime_impl_generics,
            type_impl_generics,
            lifetime_type_generics,
            type_type_generics,
            where_clause,
            ..
        } = &self.origin_generics;
        let name = &self.item.ident;
        let members = self.item.fields.members();
        let defaults = self.extra_infos.iter().map(|i| &i.default_value);

        let need_init_type = quote! {irisia::element::deps::NeedInit};
        let need_init_types = repeat(&need_init_type).take(
            self.extra_infos
                .iter()
                .filter(|i| i.generic_type.is_some())
                .count(),
        );

        quote! {
            impl <#lifetime_impl_generics #type_impl_generics> ::std::default::Default
                for #name<#lifetime_type_generics #type_type_generics #(#need_init_types,)*>
            #where_clause
            {
                fn default() -> Self {
                    Self {
                        #(#members: #defaults,)*
                    }
                }
            }
        }
    }

    pub fn generate(&self) -> TokenStream {
        [self.item.to_token_stream(), self.impl_default()]
            .into_iter()
            .collect()
    }
}

fn format_insert_generic(field: &Field) -> Result<Ident> {
    let ident = field
        .ident
        .as_ref()
        .ok_or_else(|| Error::new_spanned(field, "should be a nammed field"))?;
    let out = format_ident!("Prop{}", ident.to_string().to_camel(), span = ident.span());
    Ok(out)
}

enum Behavior {
    Required,
    Optional(Option<Expr>),
}

fn parse_and_remove_props_attr(field: &mut Field) -> Result<Behavior> {
    let r = ParseArgs::new()
        .meta(
            conflicts((
                ("default", path_only()).map(|v| {
                    if v {
                        Behavior::Optional(None)
                    } else {
                        Behavior::Required
                    }
                }),
                ("default", key_value::<Expr>()).map(|v| Behavior::Optional(Some(v))),
            ))
            .optional(),
        )
        .parse_concat_attrs(find_attr::all(&field.attrs, "props"))
        .map(|result| result.meta.unwrap_or(Behavior::Required))?;

    field.attrs.retain(|a| !a.path().is_ident("props"));
    Ok(r)
}
