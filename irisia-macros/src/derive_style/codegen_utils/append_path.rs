use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, Index, Member, Token, Type, TypeTuple};

use crate::derive_style::variant_analyzer::{
    full_quality_paths::FullQualitySegment, VariantAnalysis,
};

use super::CodeGenerator;

impl CodeGenerator {
    pub fn append_path(&mut self, va: &VariantAnalysis) {
        let variant = self.variant.clone();

        for path in va.paths.iter() {
            let (ty, body) = impl_one_path(path);
            let body = va.field_type.surround(body.into_iter());

            self.impl_trait(
                quote!(std::convert::From<#ty>),
                quote! {
                    fn from(value: #ty) -> Self {
                        #variant #body
                    }
                },
            )
        }
    }
}

fn impl_one_path(
    path: &[(Member, FullQualitySegment)],
) -> (TypeTuple, Vec<(&Member, TokenStream)>) {
    let mut elems: Punctuated<Type, Token![,]> = Punctuated::new();
    let mut output = Vec::new();

    for (tag, seg) in path {
        let tuple_index = elems.len();
        let value = match seg {
            FullQualitySegment::Required(ty) => {
                elems.push(ty.clone());
                let tuple_index: Index = tuple_index.into();
                quote!(value.#tuple_index)
            }
            FullQualitySegment::Fn { fn_path, arg_types } => {
                elems.extend(arg_types.iter().cloned());
                let index_iter = (tuple_index..tuple_index + arg_types.len()).map(Index::from);
                quote!(#fn_path(#(value.#index_iter),*))
            }
            FullQualitySegment::Default(def) => def.to_token_stream(),
        };

        output.push((tag, value));
    }

    (
        TypeTuple {
            elems,
            paren_token: Default::default(),
        },
        output,
    )
}
