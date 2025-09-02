use case::CaseExt;
use darling::{FromDeriveInput, FromField, Result as DarlingResult, ast, util::Flag};
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{DeriveInput, Generics, Ident, LitStr, Token};

use crate::{consts::*, generics_unbracketed::split_for_impl_unbracketed};

const_quote! {
    const PATH_PROPERTY = {
        #PATH_COMPONENT::property
    };
}

type MacroFields = ast::Data<(), FieldOpts>;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(prop), supports(struct_named))]
struct MacroOpts {
    ident: syn::Ident,
    generics: syn::Generics,
    data: MacroFields,
}

#[derive(Debug, FromField)]
#[darling(attributes(prop))]
struct FieldOpts {
    ident: Option<syn::Ident>,
    ty: syn::Type,

    extend: Flag,

    #[darling(default)]
    rename: Option<LitStr>,
}

impl MacroOpts {
    fn data(&self) -> &Vec<FieldOpts> {
        match &self.data {
            ast::Data::Struct(fields) => &fields.fields,
            _ => unreachable!(),
        }
    }
}

pub fn derive_prop(input: DeriveInput) -> TokenStream {
    let opts = match MacroOpts::from_derive_input(&input) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };

    let mut tokens = opts.make_template();
    tokens
}

impl MacroOpts {
    fn make_template(&self) -> TokenStream {
        let Self {
            ident: orig_struct_name,
            generics:
                orig_generics @ Generics {
                    params: orig_generics_params,
                    where_clause,
                    ..
                },
            ..
        } = self;

        let field_names = self.data().iter().map(|x| x.ident.as_ref().unwrap());

        let new_field_generics: Vec<Ident> = self
            .data()
            .iter()
            .map(FieldOpts::get_generic_name)
            .collect();

        let new_struct_name = format_ident!(
            "__IrisiaProp{orig_struct_name}",
            span = orig_struct_name.span()
        );

        let g_comma = (!orig_generics_params.empty_or_trailing()).then(<Token![,]>::default);
        let (_, orig_struct_type_generics, _) = orig_generics.split_for_impl();

        quote! {
            #[doc(hidden)]
            pub struct #new_struct_name<#orig_generics_params #g_comma #(#new_field_generics,)*>
            #where_clause
            {
                __irisia_phantom: ::core::marker::PhantomData<
                    #orig_struct_name #orig_struct_type_generics
                >,
                #(#field_names: #new_field_generics,)*
            }
        }
    }
}

impl FieldOpts {
    fn get_generic_name(&self) -> Ident {
        let ident = self.ident.as_ref().unwrap();
        let camel_case = ident.to_string().to_camel();
        format_ident!("__IrisiaType{camel_case}", span = ident.span())
    }
}
