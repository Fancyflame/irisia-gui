use std::borrow::Cow;

use attr_parser_fn::{
    find_attr,
    meta::{key_str, key_value, ParseMetaExt},
    ParseArgs, ParseAttrTrait,
};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    Attribute, Data, DeriveInput, Error, Fields, FieldsNamed, Generics, Ident, LitBool, Result,
};

use self::field::FieldProps;

mod field;

pub struct DeriveProps {
    ident: Ident,
    generics: Generics,
    struct_attr: StructAttr,
    props: Vec<FieldProps>,
}

struct StructAttr {
    rename: Option<Ident>,
    impl_from: bool,
}

impl Parse for DeriveProps {
    fn parse(input: ParseStream) -> Result<Self> {
        let input: DeriveInput = input.parse()?;

        let fields = match input.data {
            Data::Struct(s) => s.fields,
            _ => return Err(Error::new_spanned(input, "only struct is supported")),
        };

        let props = match fields {
            Fields::Unnamed(f) => {
                return Err(Error::new_spanned(f, "fields are expected to have name"))
            }
            Fields::Named(FieldsNamed { named: fields, .. }) => {
                let mut vec = Vec::new();
                for f in fields {
                    vec.push(FieldProps::parse(f)?);
                }
                vec
            }
            Fields::Unit => Vec::new(),
        };

        let struct_attr = parse_struct_attr(&input.attrs)?;

        Ok(DeriveProps {
            ident: input.ident,
            generics: input.generics,
            struct_attr,
            props,
        })
    }
}

fn parse_struct_attr(attrs: &[Attribute]) -> Result<StructAttr> {
    let Some(attr) = find_attr::only(attrs, "props")? else {
        return Ok(StructAttr {
            rename: None,
            impl_from: true,
        });
    };

    let (rename, impl_from) = ParseArgs::new()
        .meta((
            ("name", key_str::<Ident>()).optional(),
            ("impl_from", key_value::<LitBool>()).optional(),
        ))
        .parse_attrs(attr)?
        .meta;

    Ok(StructAttr {
        rename,
        impl_from: match impl_from {
            Some(b) => b.value,
            None => true,
        },
    })
}

impl DeriveProps {
    pub fn compile(&self) -> TokenStream {
        let mut tokens: TokenStream = [
            self.make_props_struct(),
            self.make_impl_default(),
            self.make_impl_user_props(),
        ]
        .into_iter()
        .collect();

        if self.struct_attr.impl_from {
            tokens.extend(self.make_impl_from());
        }

        tokens
    }

    fn struct_name(&self) -> Cow<Ident> {
        match &self.struct_attr.rename {
            Some(n) => Cow::Borrowed(n),
            None => Cow::Owned(format_ident!("{}UserProps", self.ident).into()),
        }
    }

    fn make_props_struct(&self) -> TokenStream {
        let struct_name = self.struct_name();

        let generics = &self.generics;
        let where_clause = &self.generics.where_clause;
        let fields = self.props.iter().map(FieldProps::new_field);

        quote! {
            pub struct #struct_name #generics
            #where_clause
            {
                #(pub #fields,)*
            }
        }
    }

    fn make_impl_default(&self) -> TokenStream {
        let struct_name = self.struct_name();

        let (impl_g, type_g, where_clause) = self.generics.split_for_impl();
        let fields = self.props.iter().map(FieldProps::default_field);

        quote! {
            impl #impl_g ::std::default::Default for #struct_name #type_g
            #where_clause
            {
                fn default() -> Self {
                    Self {
                        #(#fields,)*
                    }
                }
            }
        }
    }

    fn make_impl_user_props(&self) -> TokenStream {
        let host_name = &self.ident;
        let struct_name = self.struct_name();

        let (impl_g, type_g, where_clause) = self.generics.split_for_impl();
        let fields = self.props.iter().map(FieldProps::take_field);

        quote! {
            impl #impl_g ::irisia::element::UserProps for #host_name #type_g
            #where_clause
            {
                type Props = #struct_name #type_g;

                fn take(
                    #[allow(unused_variables)]
                    props: Self::Props,
                ) -> Self {
                    Self {
                        #(#fields,)*
                    }
                }
            }
        }
    }

    fn make_impl_from(&self) -> TokenStream {
        let host_name = &self.ident;
        let struct_name = self.struct_name();

        let (impl_g, type_g, where_clause) = self.generics.split_for_impl();

        quote! {
            impl #impl_g ::std::convert::From<#struct_name #type_g>
                for #host_name #type_g
                #where_clause
            {
                fn from(from: #struct_name #type_g) -> Self {
                    ::irisia::element::UserProps::take(from)
                }
            }
        }
    }
}
