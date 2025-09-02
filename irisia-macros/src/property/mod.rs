use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{DeriveInput, Generics, Ident, Result, Token, Type};

use crate::{
    consts::*, generics_unbracketed::split_for_impl_unbracketed, property::parse::parse_derive,
};

mod parse;

const_quote! {
    const PATH_PROPERTY = {
        #PATH_COMPONENT::property
    };

    const PHANTOM_DATA = {
        ::core::marker::PhantomData
    };

    const TSOME = {
        #PATH_PROPERTY::type_option::TSome
    };

    const TNONE = {
        #PATH_PROPERTY::type_option::TNone
    };
}

pub fn derive_prop(input: DeriveInput) -> Result<TokenStream> {
    let input = parse_derive(input)?;
    let tokens = [
        input.make_prop_struct(),
        input.make_mutator(),
        input.make_impl_merge_prop(),
    ];

    Ok(tokens.into_iter().collect())
}

struct MacroInput {
    ident: Ident,
    generics: Generics,
    prop_ident: Ident,
    extend_field_index: Option<usize>,
    fields: Vec<FieldInput>,
}

struct FieldInput {
    ident: Ident,
    ty: Type,
    rename: Option<Ident>,
    prop_type: Ident,
}

impl MacroInput {
    fn make_prop_struct(&self) -> TokenStream {
        let Self {
            ident: orig_struct_name,
            generics: orig_generics @ Generics { where_clause, .. },
            prop_ident,
            ..
        } = self;

        let field_names = self.fields.iter().map(|x| &x.ident);
        let new_field_generics = self.fields.iter().map(|x| &x.prop_type);
        let new_field_generics2 = new_field_generics.clone();

        let orig_generics_unbracketed = UnbracketedGenerics(orig_generics);
        let (_, orig_struct_type_generics, _) = orig_generics.split_for_impl();

        quote! {
            #[doc(hidden)]
            pub struct #prop_ident<#orig_generics_unbracketed #(#new_field_generics,)*>
            #where_clause
            {
                __irisia_phantom: #PHANTOM_DATA<#orig_struct_name #orig_struct_type_generics>,
                #(#field_names: #new_field_generics2,)*
            }
        }
    }

    fn make_mutator(&self) -> TokenStream {
        let Self {
            ident: orig_struct_name,
            generics,
            ..
        } = self;

        let mutator_name = format_ident!(
            "__IrisiaMutator{}",
            orig_struct_name,
            span = orig_struct_name.span()
        );

        let (impl_g, type_g, where_clause) = generics.split_for_impl();
        let functions = (0..self.fields.len()).map(|i| self.make_mutator_set_prop(i));

        quote! {
            pub struct #mutator_name #generics(#PHANTOM_DATA<#orig_struct_name #type_g>);
            impl #impl_g #PATH_PROPERTY::PropertyMutator for #mutator_name #type_g
            #where_clause {
                const GET: Self = Self(#PHANTOM_DATA);

            }

            impl #impl_g #mutator_name #type_g
            #where_clause
            {
                #(#functions)*
            }
        }
    }

    fn make_mutator_set_prop(&self, index: usize) -> TokenStream {
        let Self {
            generics,
            prop_ident: prop_struct_ident,
            fields: all_fields,
            ..
        } = self;

        let FieldInput {
            ident, ty, rename, ..
        } = &all_fields[index];

        let (_, generics, _) = split_for_impl_unbracketed(generics);
        let function_name = rename.as_ref().unwrap_or(ident);

        let tsome_type = quote! {#TSOME<#ty>};

        let return_prop_generics = (0..all_fields.len()).map(|i| {
            if i == index {
                &tsome_type as &dyn ToTokens
            } else {
                &TNONE
            }
        });

        let init_prop_fields = all_fields.iter().enumerate().map(|(i, f)| {
            let ident = &f.ident;
            if i == index {
                quote! {#ident: #TSOME(value)}
            } else {
                quote! {#ident: #TNONE}
            }
        });

        quote! {
            pub fn #function_name(value: #ty) -> #prop_struct_ident<
                #generics #(#return_prop_generics,)*
            > {
                #prop_struct_ident {
                    __irisia_phantom: #PHANTOM_DATA,
                    #(#init_prop_fields,)*
                }
            }
        }
    }

    fn make_impl_merge_prop(&self) -> TokenStream {
        let Self {
            generics: orig_generics,
            prop_ident,
            fields,
            ..
        } = self;

        // Original generics split
        let (orig_impl_g, orig_type_g, orig_where_bounds) =
            split_for_impl_unbracketed(orig_generics);

        // Create side generic idents for src and other for each field
        let self_generics: Vec<Ident> = (0..fields.len())
            .map(|i| format_ident!("__IrisiaS{}", i + 1))
            .collect();
        let other_generics: Vec<Ident> = (0..fields.len())
            .map(|i| format_ident!("__IrisiaO{}", i + 1))
            .collect();

        // Where predicates: each O_i: TypeOption<FieldTy>
        let where_bounds: Vec<TokenStream> = fields
            .iter()
            .zip(other_generics.iter())
            .map(|(f, o)| {
                let ty = &f.ty;
                quote! { #o: #PATH_PROPERTY::type_option::TypeOption<#ty> }
            })
            .collect();

        // Output generics: O_i::OrOutput<S_i>
        let output_generics = self_generics
            .iter()
            .zip(other_generics.iter())
            .map(|(s, o)| {
                quote! { #o::OrOutput<#s> }
            });

        // Field construction: self.field.or(other.field)
        let field_inits = fields.iter().map(|f| {
            let ident = &f.ident;
            quote! { #ident: other.#ident.or(self.#ident) }
        });

        quote! {
            impl <#orig_impl_g #(#self_generics,)* #(#other_generics,)*>
                #PATH_PROPERTY::MergePropertiesFrom<
                    #prop_ident<#orig_type_g #(#other_generics,)*>
                > for #prop_ident<#orig_type_g #(#self_generics,)*>
            where
                #orig_where_bounds
                #(#where_bounds,)*
            {
                type Output = #prop_ident<
                    #orig_type_g
                    #(#output_generics,)*
                >;

                fn merge(self, other: #prop_ident<#orig_type_g #(#other_generics,)*>) -> Self::Output {
                    #prop_ident {
                        __irisia_phantom: #PHANTOM_DATA,
                        #(#field_inits,)*
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct UnbracketedGenerics<'a>(&'a Generics);

impl ToTokens for UnbracketedGenerics<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let params = &self.0.params;
        params.to_tokens(tokens);
        if !params.empty_or_trailing() {
            <Token![,]>::default().to_tokens(tokens);
        }
    }
}
