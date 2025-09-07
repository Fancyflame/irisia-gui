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
}

pub fn derive_prop(input: DeriveInput) -> Result<TokenStream> {
    let input = parse_derive(input)?;
    let tokens = [
        input.make_prop_struct(),
        input.impl_prop_empty(),
        input.make_mutator(),
        input.impl_prop_update(),
        input.impl_prop_cast(),
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
        let new_field_generics: Vec<&Ident> = self.fields.iter().map(|x| &x.prop_type).collect();

        let orig_generics_unbracketed = UnbracketedGenerics(orig_generics);
        let (_, orig_struct_type_generics, _) = orig_generics.split_for_impl();

        quote! {
            #[doc(hidden)]
            pub struct #prop_ident<#orig_generics_unbracketed #(#new_field_generics,)*>
            #where_clause
            {
                __irisia_phantom: #PHANTOM_DATA<#orig_struct_name #orig_struct_type_generics>,
                #(#field_names: #new_field_generics,)*
            }
        }
    }

    fn impl_prop_empty(&self) -> TokenStream {
        let Self {
            ident, prop_ident, ..
        } = self;

        let (impl_g, type_g, where_clause) = split_for_impl_unbracketed(&self.generics);
        let field_types = self.fields.iter().map(|f| &f.ty);
        let field_init = self.fields.iter().map(|f| {
            let FieldInput { ident, ty, .. } = f;
            quote! {
                #ident: <#ty as #PATH_PROPERTY::PropEmpty>::EMPTY,
            }
        });

        let path_property = PATH_PROPERTY;
        quote! {
            impl<#impl_g> #PATH_PROPERTY::PropEmpty for #ident<#type_g>
            #where_clause
            {
                type Empty = #prop_ident<
                    #type_g
                    #(<#field_types as #path_property::PropEmpty>::Empty,)*
                >;

                const EMPTY: Self::Empty = Self::Empty {
                    __irisia_phantom: #PHANTOM_DATA,
                    #(#field_init)*
                };
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
        let functions = (0..self.fields.len()).map(|i| self.make_mutator_function(i));

        quote! {
            #[doc(hidden)]
            pub struct #mutator_name #generics(#PHANTOM_DATA<#orig_struct_name #type_g>);

            impl #impl_g #PATH_PROPERTY::PropertyMutator for #orig_struct_name #type_g
            #where_clause {
                type Mutator = #mutator_name #type_g;
                const GET: Self::Mutator = #mutator_name(#PHANTOM_DATA);
            }

            impl #impl_g #mutator_name #type_g
            #where_clause
            {
                #(#functions)*
            }
        }
    }

    fn make_mutator_function(&self, index: usize) -> TokenStream {
        let Self {
            generics,
            prop_ident: prop_struct_ident,
            fields: all_fields,
            ..
        } = self;

        let FieldInput { ident, rename, .. } = &all_fields[index];

        let (_, generics, _) = split_for_impl_unbracketed(generics);
        let function_name = rename.as_ref().unwrap_or(ident);

        let return_prop_types = all_fields
            .iter()
            .enumerate()
            .map(|(i, FieldInput { ty, .. })| {
                if i == index {
                    quote! {__IrisiaValue}
                } else {
                    quote! {<#ty as #PATH_PROPERTY::PropEmpty>::Empty}
                }
            });

        let init_prop_fields = all_fields
            .iter()
            .enumerate()
            .filter(|&(i, _)| i != index)
            .map(|(_, f)| {
                let FieldInput { ident, ty, .. } = f;
                quote! {
                    #ident: <#ty as #PATH_PROPERTY::PropEmpty>::EMPTY,
                }
            });

        quote! {
            pub fn #function_name<__IrisiaValue>(&self, value: __IrisiaValue)
                -> #prop_struct_ident<#generics #(#return_prop_types,)*>
            // where
                // #ty: #PATH_PROPERTY::PropUpdate<#prop_type, __IrisiaValue>,
            {
                #prop_struct_ident {
                    __irisia_phantom: #PHANTOM_DATA,
                    #ident: value,
                    #(#init_prop_fields)*
                }
            }
        }
    }

    fn impl_prop_update(&self) -> TokenStream {
        let Self {
            ident: orig_struct_name,
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
            .map(|i| format_ident!("__IrisiaS{i}"))
            .collect();
        let other_generics: Vec<Ident> = (0..fields.len())
            .map(|i| format_ident!("__IrisiaO{i}"))
            .collect();

        let output_generics = self_generics
            .iter()
            .zip(other_generics.iter())
            .zip(fields.iter())
            .map(|((s, o), FieldInput { ty, .. })| {
                quote! {
                    <#ty as #PATH_PROPERTY::PropUpdate<#s, #o>>::Output,
                }
            });

        // Field construction: self.field.or(other.field)
        let field_inits = fields.iter().map(|FieldInput { ident, ty, .. }| {
            quote! {
                #ident: <#ty as #PATH_PROPERTY::PropUpdate<_, _>>::prop_update(
                    this.#ident,
                    other.#ident,
                ),
            }
        });

        let field_types = fields.iter().map(|f| &f.ty);

        let path_property = PATH_PROPERTY;
        quote! {
            impl <#orig_impl_g #(#self_generics,)* #(#other_generics,)*>
                #PATH_PROPERTY::PropUpdate<
                    #prop_ident<#orig_type_g #(#self_generics,)*>,
                    #prop_ident<#orig_type_g #(#other_generics,)*>,
                > for #orig_struct_name<#orig_type_g>
            where
                #orig_where_bounds
                #(#field_types: #path_property::PropUpdate<
                    #self_generics,
                    #other_generics,
                >,)*
            {
                type Output = #prop_ident<
                    #orig_type_g
                    #(#output_generics)*
                >;

                fn prop_update(
                    this: #prop_ident<#orig_type_g #(#self_generics,)*>,
                    other: #prop_ident<#orig_type_g #(#other_generics,)*>
                ) -> Self::Output {
                    #prop_ident {
                        __irisia_phantom: #PHANTOM_DATA,
                        #(#field_inits)*
                    }
                }
            }
        }
    }

    fn impl_prop_cast(&self) -> TokenStream {
        let Self {
            ident: orig_ident,
            prop_ident,
            ..
        } = self;

        let (impl_g, type_g, where_bounds) = split_for_impl_unbracketed(&self.generics);
        let field_generics: Vec<&Ident> = self.fields.iter().map(|f| &f.prop_type).collect();

        let field_where_bounds = self.fields.iter().map(|f| {
            let FieldInput { ty, prop_type, .. } = f;
            quote! {
                #ty: #PATH_PROPERTY::PropCast<#prop_type>,
            }
        });

        let final_struct_init_field = self.fields.iter().map(|f| {
            let ident = &f.ident;
            quote! {
                #ident: #PATH_PROPERTY::PropCast::prop_cast(value.#ident),
            }
        });

        let value_prop = quote! {
            #prop_ident<#type_g #(#field_generics,)*>
        };

        quote! {
            impl<#impl_g #(#field_generics,)*> #PATH_PROPERTY::PropCast<#value_prop>
                for #orig_ident<#type_g>
            where
                #where_bounds
                #(#field_where_bounds)*
            {
                fn prop_cast(value: #value_prop) -> Self {
                    Self {
                        #(#final_struct_init_field)*
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
