use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Type};

use crate::component::{
    codegen::{PATH_COMPONENT, STRUCT_STORAGE},
    DomMacro, FieldType,
};

pub struct CompStorage<'a> {
    d: &'a DomMacro,
    user_data: HashMap<&'a Ident, &'a Type>,
}

impl<'a> CompStorage<'a> {
    pub fn new(d: &'a DomMacro) -> Self {
        let mut user_data = HashMap::new();
        for field in d.fields.iter() {
            match &field.field_type {
                FieldType::Value { to_ty, .. } if !matches!(to_ty, Type::Infer(_)) => {
                    user_data.insert(&field.name, to_ty);
                }
                _ => {}
            }
        }

        Self { d, user_data }
    }

    pub fn gen_storage_def(&self) -> TokenStream {
        let DomMacro { generics, .. } = self.d;

        let user_data_fields = self.user_data.iter().map(|(name, ty)| {
            let name = user_data_key(name);
            quote! {
                #name: #PATH_COMPONENT::field::FieldHook<#ty>,
            }
        });

        quote! {
            pub struct #STRUCT_STORAGE #generics {
                #(#user_data_fields)*
                child_box: #PATH_COMPONENT::child_box::ChildBox,
            }
        }
    }
}

pub fn user_data_key(i: &Ident) -> Ident {
    format_ident!("user_data_{i}")
}

pub fn child_box_key() -> Ident {
    format_ident!("child_box")
}
