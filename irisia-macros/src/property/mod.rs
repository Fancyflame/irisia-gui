use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{DeriveInput, Generics, Ident, Result, Token, Type, Visibility};

use crate::{
    consts::*, generics_unbracketed::split_for_impl_unbracketed, property::parse::parse_derive,
};

mod parse;

const_quote! {
    const PHANTOM_DATA = {
        ::core::marker::PhantomData
    };
}

pub fn derive_prop(input: DeriveInput) -> Result<TokenStream> {
    let input = parse_derive(input)?;
    let tokens = [
        input.make_prop_struct(),
        input.impl_property(),
        input.make_prop_mutator(),
        input.impl_prop_update(),
        input.impl_prop_cast(),
        input.make_extend(),
    ];

    Ok(tokens.into_iter().collect())
}

struct MacroInput {
    vis: Visibility,
    ident: Ident,
    generics: Generics,
    prop_ident: Ident,
    mutator_ident: Ident,
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
            vis,
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
            #vis struct #prop_ident<#orig_generics_unbracketed #(#new_field_generics,)*>
            #where_clause
            {
                __irisia_phantom: #PHANTOM_DATA<#orig_struct_name #orig_struct_type_generics>,
                #(#field_names: #new_field_generics,)*
            }
        }
    }

    fn impl_property(&self) -> TokenStream {
        let Self {
            mutator_ident,
            ident: orig_struct_name,
            generics,
            ..
        } = self;
        let (impl_g, type_g, where_clause) = generics.split_for_impl();
        let empty_dec = self.make_empty_dec();

        quote! {
            impl #impl_g #PATH_PROPERTY::Property for #orig_struct_name #type_g
            #where_clause {
                type Mutator = #mutator_ident #type_g;
                const MUTATOR: Self::Mutator = #mutator_ident(#PHANTOM_DATA);

                #empty_dec
            }
        }
    }

    fn make_empty_dec(&self) -> TokenStream {
        let Self { prop_ident, .. } = self;

        let (_, type_g, _) = split_for_impl_unbracketed(&self.generics);
        let field_types = self.fields.iter().map(|f| &f.ty);
        let field_init = self.fields.iter().map(|f| {
            let FieldInput { ident, ty, .. } = f;
            quote! {
                #ident: <#ty as #PATH_PROPERTY::Property>::EMPTY,
            }
        });

        let path_property = PATH_PROPERTY;
        quote! {
            type Empty = #prop_ident<
                #type_g
                #(<#field_types as #path_property::Property>::Empty,)*
            >;

            const EMPTY: Self::Empty = Self::Empty {
                __irisia_phantom: #PHANTOM_DATA,
                #(#field_init)*
            };
        }
    }

    fn make_prop_mutator(&self) -> TokenStream {
        let Self {
            vis,
            ident: orig_struct_name,
            generics,
            mutator_ident,
            ..
        } = self;

        let (impl_g, type_g, where_clause) = generics.split_for_impl();
        let functions = (0..self.fields.len()).map(|i| self.make_mutator_function(i));

        quote! {
            #[doc(hidden)]
            #vis struct #mutator_ident #generics(#PHANTOM_DATA<#orig_struct_name #type_g>);

            impl #impl_g #mutator_ident #type_g
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
                    quote! {<#ty as #PATH_PROPERTY::Property>::Empty}
                }
            });

        let init_prop_fields = all_fields
            .iter()
            .enumerate()
            .filter(|&(i, _)| i != index)
            .map(|(_, f)| {
                let FieldInput { ident, ty, .. } = f;
                quote! {
                    #ident: <#ty as #PATH_PROPERTY::Property>::EMPTY,
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

        let (orig_impl_g, orig_type_g, orig_where_bounds) =
            split_for_impl_unbracketed(orig_generics);

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

    fn make_extend(&self) -> TokenStream {
        let (
            extend_field_index,
            FieldInput {
                ident: field_ident,
                ty: field_type,
                prop_type: field_generic_type,
                ..
            },
        ) = match self.extend_field_index {
            Some(index) => (index, &self.fields[index]),
            None => return TokenStream::new(),
        };

        let Self {
            ident,
            mutator_ident,
            fields,
            prop_ident,
            generics,
            ..
        } = self;

        let (impl_g, type_g, where_bounds) = split_for_impl_unbracketed(generics);
        let field_generics: Vec<&Ident> = fields.iter().map(|f| &f.prop_type).collect();

        let return_field_generics = field_generics.iter().enumerate().map(|(i, &g)| {
            if i == extend_field_index {
                quote! {
                    <#field_type as #PATH_PROPERTY::PropUpdate<
                        #field_generic_type,
                        __IrisiaUpdate,
                        __IrisiaSourceFrom,
                    >>::Output,
                }
            } else {
                quote! { #g, }
            }
        });

        let other_field_idents = self.fields.iter().enumerate().filter_map(|(i, f)| {
            if i == extend_field_index {
                None
            } else {
                Some(&f.ident)
            }
        });

        let this_type = quote! {
            #prop_ident<#type_g #(#field_generics,)*>
        };

        quote! {
            impl<#impl_g> ::core::ops::Deref for #mutator_ident<#type_g>
            where
                #where_bounds
            {
                type Target = <#field_type as #PATH_PROPERTY::Property>::Mutator;
                fn deref(&self) -> &Self::Target {
                    &<#field_type as #PATH_PROPERTY::Property>::MUTATOR
                }
            }

            impl<__IrisiaUpdate, __IrisiaSourceFrom, #impl_g #(#field_generics,)*>
                #PATH_PROPERTY::PropUpdate<
                    #this_type,
                    __IrisiaUpdate,
                    (__IrisiaSourceFrom,),
                > for #ident<#type_g>
            where
                #where_bounds
                #field_type: #PATH_PROPERTY::PropUpdate<#field_generic_type, __IrisiaUpdate, __IrisiaSourceFrom>,
            {
                type Output = #prop_ident<#type_g #(#return_field_generics)*>;
                fn prop_update(this: #this_type, other: __IrisiaUpdate) -> Self::Output {
                    #prop_ident {
                        __irisia_phantom: #PHANTOM_DATA,
                        #field_ident: <#field_type as #PATH_PROPERTY::PropUpdate<_, _, _>>::prop_update(
                            this.#field_ident,
                            other,
                        ),
                        #(#other_field_idents: this.#other_field_idents,)*
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
