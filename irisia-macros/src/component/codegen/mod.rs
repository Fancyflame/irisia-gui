use case::CaseExt;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use stmt::gen_chained;
use storage::CompStorage;

use crate::component::{FieldDefinition, FieldType};

use super::DomMacro;

mod stmt;
mod storage;

const_quote! {
    const VAR_INPUT_DP = { __irisia_input_dirty_points };
    const PATH_CONTROL_FLOW = { irisia::model::control_flow };
    const PATH_COMPONENT = { irisia::model::component };
    const STRUCT_STORAGE = { __IrisiaComponentStorage };
}

impl DomMacro {
    pub fn gen_code(&self) -> TokenStream {
        Codegen::new(self).gen_code()
    }
}

struct Codegen<'a> {
    d: &'a DomMacro,
    storage: CompStorage<'a>,
}

impl<'a> Codegen<'a> {
    fn new(d: &'a DomMacro) -> Self {
        Codegen {
            d,
            storage: CompStorage::new(d),
        }
    }

    fn gen_code(&self) -> TokenStream {
        let DomMacro {
            name,
            generics,
            fields,
            body,
        } = self.d;
        let snake_case = format_ident!("{}", name.to_string().to_snake());
        let storage_struct = self.storage.gen_storage_def();

        quote! {
            pub mod #snake_case {
                use super::*;

                pub use __irisia_comp::#STRUCT_STORAGE as Storage;

                mod __irisia_comp {
                    use super::*;

                    #storage_struct
                }
            }

        }
    }

    fn gen_storage_struct(&self) -> TokenStream {
        let DomMacro {
            generics, fields, ..
        } = self.d;

        let fields = fields
            .iter()
            .filter_map(|FieldDefinition { name, field_type }| {
                let ty = match field_type {
                    FieldType::Model => return None,
                    FieldType::Value(ty) => ty,
                };
                Some(quote! { #name: #ty, })
            });

        quote! {
            pub struct __IrisiaComponentStorage #generics {
                #(#fields)*
            }
        }
    }
}
