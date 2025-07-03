use attr_parser_fn::{ParseArgs, ParseAttrTrait, find_attr, meta::path_only};
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Error, Field, Fields, Ident, ItemStruct, Result, Type,
    parse::{Parse, ParseStream, Parser},
    parse_quote,
};

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

        let use_style_trait = quote! {
            ::irisia::model::UseStyle
        };

        let mut trait_generics = generics.clone();
        trait_generics.make_where_clause().predicates.push({
            let (_, type_g, _) = generics.split_for_impl();
            parse_quote!(
                Self: #use_style_trait<#struct_name #type_g> + ::core::marker::Sized
            )
        });

        let (impl_g, type_g, where_clause) = trait_generics.split_for_impl();

        let mut added_self_generics = trait_generics.clone();
        added_self_generics.params.push(parse_quote!(__IrisiaSelf));
        let (impl_g_with_self, _, _) = added_self_generics.split_for_impl();

        let field_idents = struct_def
            .fields
            .iter()
            .map(|field| field.ident.as_ref().unwrap());
        let field_types = struct_def.fields.iter().map(|field| &field.ty);

        quote! {
            #struct_def

            #vis trait #trait_name #trait_generics
            #where_clause
            {
                #(
                    fn #field_idents(mut self, value: #field_types) -> Self {
                        #use_style_trait::style_mut(&mut self).#field_idents = value;
                        self
                    }
                )*
            }

            impl #impl_g_with_self #trait_name #type_g
                for __IrisiaSelf #where_clause
            {}
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
