use darling::{FromDeriveInput, FromField, Result as DarlingResult, ast, util::Flag};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{DeriveInput, Generics, Ident, LitStr};

use crate::consts::*;

const_quote! {
    const PATH_PROPERTY = {
        #PATH_COMPONENT::property
    };
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(prop), supports(struct_named))]
struct MacroOpts {
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<(), FieldOpts>,
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

pub fn derive_prop(input: DeriveInput) -> TokenStream {
    let opts = match MacroOpts::from_derive_input(&input) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };

    let field_cfgs: Vec<FieldOpts> = match opts.data {
        ast::Data::Struct(fields) => fields.fields,
        _ => unreachable!(),
    };

    todo!();
}
