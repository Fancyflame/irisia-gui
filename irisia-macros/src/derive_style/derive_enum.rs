use std::borrow::Cow;

use case::CaseExt;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Data, DataEnum, DeriveInput, Error, Ident, Member, Result, Token, Variant};

use super::{
    codegen_utils::CodeGenerator,
    variant_analyzer::{read_fields::FieldAnalysis, VariantAnalysis},
};

pub fn derive_style_for_enum(input: DeriveInput) -> Result<TokenStream> {
    let mut tokens = {
        let mut codegen = CodeGenerator::new(input.ident.clone(), None, input.generics.clone());
        codegen.impl_style();
        codegen.finish()
    };

    match input.data {
        Data::Enum(DataEnum { variants, .. }) => {
            for var in variants.into_iter() {
                let mut codegen = CodeGenerator::new(
                    input.ident.clone(),
                    Some(&var.ident),
                    input.generics.clone(),
                );
                handle_variant(&mut codegen, &var)?;
                codegen.finish().to_tokens(&mut tokens);
            }
        }
        _ => unreachable!("expected `DataEnum`, found {:?}", input.data),
    }

    Ok(tokens)
}

fn handle_variant(codegen: &mut CodeGenerator, variant: &Variant) -> Result<()> {
    let va = VariantAnalysis::analyze_fields(&variant.attrs, &variant.fields)?;

    append_variant_option(codegen, &variant.ident, &va)?;
    if va.impl_default {
        codegen.impl_default(&va)?;
    }
    codegen.append_path(&va);
    for field in va.fields.values() {
        try_append_option(codegen, &variant.ident, field);
    }

    Ok(())
}

fn try_append_option(codegen: &mut CodeGenerator, variant_name: &Ident, field: &FieldAnalysis) {
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

    let pattern = match tag {
        Member::Named(name) => quote!(Self::#variant_name { #name: origin, .. }),
        Member::Unnamed(index) => {
            let omitted = std::iter::repeat_with(<Token![_]>::default).take(index.index as _);
            quote!(Self::#variant_name( #(#omitted,)* origin, .. ))
        }
    };

    let panic_msg = format!("this function is valid only when self is `{variant_name}`");

    let body = if *option_set_true {
        quote! {
            pub fn #fn_name(&mut self) {
                match self {
                    #pattern => *origin = true,
                    _ => panic!(#panic_msg)
                }
            }
        }
    } else {
        quote! {
            pub fn #fn_name(&mut self, value: #ty) {
                match self {
                    #pattern => *origin = value,
                    _ => panic!(#panic_msg)
                }
            }
        }
    };

    codegen.append_fn(body);
}

fn append_variant_option(
    codegen: &mut CodeGenerator,
    variant_name: &Ident,
    va: &VariantAnalysis,
) -> Result<()> {
    let fn_name = match &va.option {
        Some(rename) => match rename {
            Some(r) => Cow::Borrowed(r),
            None => Cow::Owned(Ident::new(
                &variant_name.to_string().to_snake(),
                variant_name.span(),
            )),
        },

        None => return Ok(()),
    };

    if va.fields.len() > 1 {
        return Err(Error::new(
            Span::call_site(),
            format!("cannot make variant `{variant_name}` an option, because it contains more than 1 fields"),
        ));
    }

    let func = match va.fields.iter().next() {
        None => {
            let body = va.field_type.surround::<_, TokenStream>(std::iter::empty());
            quote! {
                pub fn #fn_name(&mut self) {
                    *self = Self::#variant_name #body
                }
            }
        }
        Some((tag, field)) => {
            let ty = &field.ty;
            let body = va
                .field_type
                .surround(std::iter::once((tag, quote!(value))));

            quote! {
                pub fn #fn_name(&mut self, value: #ty) {
                    *self = Self::#variant_name #body
                }
            }
        }
    };

    codegen.append_fn(func);
    Ok(())
}
