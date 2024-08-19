use std::borrow::Cow;

use attr_parser_fn::{
    meta::{key_str, ParseMetaExt},
    ParseArgs, ParseAttrTrait,
};

use field::Defaulter;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{ParseStream, Parser},
    parse_quote, Error, Fields, FieldsNamed, Ident, Item, ItemStruct, Result,
};

use self::field::FieldProps;

mod field;

pub struct DeriveProps {
    input: ItemStruct,
    struct_attr: StructAttr,
    props: Vec<FieldProps>,
}

struct StructAttr {
    rename: Option<Ident>,
}

impl DeriveProps {
    pub fn parse(attr: TokenStream, input: Item) -> Result<Self> {
        let input = match input {
            Item::Struct(s) => s,
            _ => return Err(Error::new_spanned(input, "only struct is supported")),
        };

        let props = match &input.fields {
            Fields::Unnamed(f) => {
                return Err(Error::new_spanned(
                    f,
                    format!(
                        "fields are expected to have names. try using \
                        `struct {struct_name} {{ field: Type, ... }}` to \
                        define it",
                        struct_name = input.ident
                    ),
                ))
            }
            Fields::Named(FieldsNamed { named: fields, .. }) => {
                let mut vec = Vec::new();
                for f in fields {
                    vec.push(FieldProps::parse(f.clone())?);
                }
                vec
            }
            Fields::Unit => Vec::new(),
        };

        let struct_attr = parse_struct_attr.parse2(attr)?;

        Ok(DeriveProps {
            input,
            struct_attr,
            props,
        })
    }
}

fn parse_struct_attr(attr: ParseStream) -> Result<StructAttr> {
    let (rename,) = ParseArgs::new()
        .meta((("name", key_str::<Ident>()).optional(),))
        .parse(attr)?
        .meta;

    Ok(StructAttr { rename })
}

impl DeriveProps {
    pub fn compile(&self) -> TokenStream {
        [
            self.remake_origin_struct(),
            self.make_props_struct(),
            self.make_impl_default(),
            self.make_impl_user_props(),
        ]
        .into_iter()
        .collect()
    }

    fn struct_name(&self) -> Cow<Ident> {
        match &self.struct_attr.rename {
            Some(n) => Cow::Borrowed(n),
            None => Cow::Owned(format_ident!("{}UserProps", self.input.ident).into()),
        }
    }

    fn remake_origin_struct(&self) -> TokenStream {
        let mut item = self.input.clone();
        for (prop, field) in self.props.iter().zip(item.fields.iter_mut()) {
            let old_ty = &field.ty;
            let new_ty = parse_quote! {
                ::irisia::data_flow::ReadWire<#old_ty>
            };

            field.ty = new_ty;
            if let Defaulter::Optioned = prop.defaulter {
                let old_ty = &field.ty;
                field.ty = parse_quote! {
                    ::std::option::Option<#old_ty>
                };
            }

            field.attrs.retain(|x| !x.path().is_ident("props"));
        }
        item.into_token_stream()
    }

    fn make_props_struct(&self) -> TokenStream {
        let struct_name = self.struct_name();

        let generics = &self.input.generics;
        let where_clause = &self.input.generics.where_clause;
        let fields = self.props.iter().map(FieldProps::new_field);

        quote! {
            pub struct #struct_name #generics
            #where_clause
            {
                #(pub #fields)*
            }
        }
    }

    fn make_impl_default(&self) -> TokenStream {
        let struct_name = self.struct_name();

        let (impl_g, type_g, where_clause) = self.input.generics.split_for_impl();
        let fields = self.props.iter().map(FieldProps::default_field);

        quote! {
            impl #impl_g ::std::default::Default for #struct_name #type_g
            #where_clause
            {
                fn default() -> Self {
                    Self {
                        #(#fields)*
                    }
                }
            }
        }
    }

    fn make_impl_user_props(&self) -> TokenStream {
        let host_name = &self.input.ident;
        let struct_name = self.struct_name();

        let (impl_g, type_g, where_clause) = self.input.generics.split_for_impl();
        let fields = self.props.iter().map(FieldProps::take_field);

        quote! {
            impl #impl_g ::irisia::element::FromUserProps for #host_name #type_g
            #where_clause
            {
                type Props = #struct_name #type_g;

                fn take(
                    _props: Self::Props,
                ) -> Self {
                    Self {
                        #(#fields)*
                    }
                }
            }
        }
    }
}
