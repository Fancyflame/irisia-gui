use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Error, Result};

use super::{
    codegen_utils::CodeGenerator,
    variant_analyzer::{read_fields::FieldAnalysis, VariantAnalysis},
};

pub fn derive_style_for_struct(input: DeriveInput) -> Result<TokenStream> {
    let mut codegen = CodeGenerator::new(input.ident.clone(), None, input.generics);

    let analysis = match input.data {
        Data::Struct(DataStruct { fields, .. }) => {
            VariantAnalysis::analyze_fields(&input.attrs, &fields)?
        }
        _ => unreachable!("expected `DataStruct`, found {:?}", input.data),
    };

    if analysis.option.is_some() {
        return Err(Error::new(
            Span::call_site(),
            "`option` attribute is not allowed in item struct",
        ));
    }

    codegen.impl_style();
    codegen.append_path(&analysis);
    for field in analysis.fields.values() {
        try_append_option(&mut codegen, field);
    }

    if analysis.impl_default {
        codegen.impl_default(&analysis)?;
    }

    Ok(codegen.finish())
}

fn try_append_option(codegen: &mut CodeGenerator, field: &FieldAnalysis) {
    let FieldAnalysis {
        tag,
        ty,
        option,
        option_set_true,
        ..
    } = field;

    let Some(fn_name) = option
    else {
        return;
    };

    let body = if *option_set_true {
        quote! {
            pub fn #fn_name(&mut self) {
                self.#tag = true;
            }
        }
    } else {
        quote! {
            pub fn #fn_name(&mut self, value: #ty) {
                self.#tag = value;
            }
        }
    };

    codegen.append_fn(body);
}
