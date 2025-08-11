use std::{cell::OnceCell, iter::repeat, mem::replace};

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
    parse_quote, Error, Expr, Field, Ident, ItemStruct, Result, Type,
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
                        skip: false,
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
                    skip: false,
                },

                Behavior::Skip(default_optioned) => ExtraFieldInfo {
                    default_value: default_optioned
                        .unwrap_or_else(|| parse_quote!(::std::default::Default::default())),
                    origin_type: field.ty.clone(),
                    generic_type: None,
                    skip: true,
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
    skip: bool,
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
    Skip(Option<Expr>),
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
                ("skip", path_only()).map(|_| Behavior::Skip(None)),
                ("skip", key_value::<Expr>()).map(|v| Behavior::Skip(Some(v))),
            ))
            .optional(),
        )
        .parse_concat_attrs(find_attr::all(&field.attrs, "props"))
        .map(|result| result.meta.unwrap_or(Behavior::Required))?;

    field.attrs.retain(|a| !a.path().is_ident("props"));
    Ok(r)
}

impl CastProp {
    pub fn generate(&self) -> TokenStream {
        [
            self.item.to_token_stream(),
            self.impl_empty_props(),
            self.impl_set_fields(),
        ]
        .into_iter()
        .collect()
    }

    fn impl_empty_props(&self) -> TokenStream {
        let SplittedGenerics {
            lifetime_impl_generics,
            type_impl_generics,
            lifetime_type_generics,
            type_type_generics,
            where_clause,
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
            impl <#lifetime_impl_generics #type_impl_generics>
                irisia::element::deps::AsEmptyProps
                for #name<#lifetime_type_generics #type_type_generics>
            #where_clause
            {
                type AsEmpty = #name<
                    #lifetime_type_generics
                    #type_type_generics
                    #(#need_init_types,)*
                >;

                fn empty_props() -> Self::AsEmpty {
                    #name {
                        #(#members: #defaults,)*
                    }
                }
            }
        }
    }

    fn impl_set_fields(&self) -> TokenStream {
        let (impl_g, type_g, where_clause) = self.item.generics.split_for_impl();

        let struct_name = &self.item.ident;
        let fields = self
            .item
            .fields
            .iter()
            .enumerate()
            .map(|(index, field)| self.impl_set_field(field, index));

        quote! {
            impl #impl_g #struct_name #type_g
            #where_clause
            {
                #(#fields)*
            }
        }
    }

    fn impl_set_field(&self, field: &Field, field_index: usize) -> TokenStream {
        let ExtraFieldInfo {
            origin_type, skip, ..
        } = &self.extra_infos[field_index];

        if *skip {
            return TokenStream::new();
        }

        let as_init_type = OnceCell::new();
        // Iterator<Item = &dyn ToTokens>,
        // find out all generics and replace current generic with SimpleProvider<T>
        let extra_generics = self
            .extra_infos
            .iter()
            .enumerate()
            .filter_map(|(index, info)| {
                let t = info.generic_type.as_ref()?;
                if index == field_index {
                    Some(as_init_type.get_or_init(|| {
                        quote! {
                            irisia::hook::SimpleProvider<#origin_type>
                        }
                    }) as &dyn ToTokens)
                } else {
                    Some(t)
                }
            });

        let set_new_fields =
            self.item
                .fields
                .iter()
                .enumerate()
                .map(|(index, Field { ident, .. })| {
                    if index == field_index {
                        quote! {
                            #ident: irisia::hook::simple::IntoSimpleProvider::into_simple_provider(
                                value
                            ),
                        }
                    } else {
                        quote! { #ident: self.#ident, }
                    }
                });

        let struct_name = &self.item.ident;
        let field_name = field.ident.as_ref().unwrap();
        let SplittedGenerics {
            lifetime_type_generics,
            type_type_generics,
            ..
        } = &self.origin_generics;

        quote! {
            #[cfg(doc)]
            pub fn #field_name(
                self,
                value: impl irisia::hook::simple::IntoSimpleProvider<
                    #origin_type,
                    _,
                >
            );

            #[cfg(not(doc))]
            pub fn #field_name<__IrisiaMarker>(
                self,
                value: impl irisia::hook::simple::IntoSimpleProvider<
                    #origin_type,
                    __IrisiaMarker,
                >
            ) -> #struct_name<
                #lifetime_type_generics
                #type_type_generics
                #(#extra_generics,)*
            > {
                #struct_name {
                    #(#set_new_fields)*
                }
            }
        }
    }
}
