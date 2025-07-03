use attr_parser_fn::{ParseArgs, ParseAttrTrait, find_attr, meta::path_only};
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Error, Fields, Ident, ItemStruct, Result, Type,
    parse::{Parse, ParseStream, Parser},
};

use crate::generics_unbracketed::split_for_impl_unbracketed;

pub struct ImplStyle {
    trait_name: Ident,
    extends: Option<(Ident, Type)>,
    struct_def: ItemStruct,
}

impl ImplStyle {
    pub fn parse(attr: TokenStream1, struct_tokens: TokenStream1) -> Result<Self> {
        let ParseArgs {
            args: (trait_name,),
            ..
        } = (|input: ParseStream| ParseArgs::new().args::<(Ident,)>().parse(input)).parse(attr)?;

        let mut struct_def = ItemStruct::parse.parse(struct_tokens)?;
        let extends = find_extend_field(&mut struct_def)?;

        Ok(Self {
            trait_name,
            extends,
            struct_def,
        })
    }

    pub fn to_tokens(&self) -> TokenStream {
        let Self {
            trait_name,
            extends,
            struct_def,
        } = self;

        let ItemStruct {
            vis,
            generics,
            ident: struct_name,
            ..
        } = struct_def;

        let use_style_trait = quote! { irisia::model::UseStyle };
        let (impl_g, type_g, where_predicates) = split_for_impl_unbracketed(generics);

        let field_idents = struct_def
            .fields
            .iter()
            .map(|field| field.ident.as_ref().unwrap());
        let field_types = struct_def.fields.iter().map(|field| &field.ty);

        let extend_tokens = extends.as_ref().map(|(field_name, field_ty)| {
            quote! {
                impl<#impl_g __IrisiaExtended> #use_style_trait<__IrisiaExtended>
                    for #struct_name< #type_g >
                where
                    #where_predicates
                    #field_ty: #use_style_trait<__IrisiaExtended>,
                {
                    fn style_mut(&mut self) -> &mut __IrisiaExtended {
                        #use_style_trait::style_mut(&mut self.#field_name)
                    }
                }
            }
        });

        quote! {
            #struct_def

            #vis trait #trait_name< #type_g >
            where
                #where_predicates
                Self: #use_style_trait< #struct_name<#type_g> > + ::core::marker::Sized,
            {
                #(
                    fn #field_idents(mut self, value: #field_types) -> Self {
                        #use_style_trait::style_mut(&mut self).#field_idents = value;
                        self
                    }
                )*
            }

            impl<#impl_g __IrisiaSelf> #trait_name<#type_g> for __IrisiaSelf
            where
                #where_predicates
                Self: #use_style_trait< #struct_name<#type_g> > + ::core::marker::Sized,
            {}

            #extend_tokens
        }
    }
}

fn find_extend_field(item: &mut ItemStruct) -> Result<Option<(Ident, Type)>> {
    let fields = match &mut item.fields {
        fields @ Fields::Unnamed(_) => {
            return Err(Error::new_spanned(fields, "fields cannot be unnamed"));
        }
        Fields::Unit => return Ok(None),
        Fields::Named(fields) => fields,
    };

    let mut extends = None;
    for field in fields.named.iter_mut() {
        let Some(style_attr) = find_attr::only(&field.attrs, "style")? else {
            continue;
        };

        let ParseArgs {
            meta: should_extend,
            ..
        } = ParseArgs::new()
            .meta(("extend", path_only()))
            .parse_attr(style_attr)?;

        if should_extend {
            if extends.is_some() {
                return Err(Error::new_spanned(
                    style_attr,
                    "one style struct can only have one extend field",
                ));
            } else {
                extends = Some((field.ident.clone().unwrap(), field.ty.clone()))
            }
        }

        field.attrs.retain(|attr| !attr.path().is_ident("style"));
    }

    Ok(extends)
}
