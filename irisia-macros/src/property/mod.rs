use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{DeriveInput, Generics, Ident, Result, Token, Type, Visibility};

use crate::{
    consts::*,
    generics_unbracketed::{WhereClausePredicates, split_for_impl_unbracketed},
    property::parse::parse_derive,
};

mod parse;

const_quote! {
    const PHANTOM_DATA = {
        ::core::marker::PhantomData
    };

    const MACRO_UTILS = {
        #PATH_PROPERTY::macro_utils
    };
}

pub fn derive_prop(input: DeriveInput) -> Result<TokenStream> {
    let input = parse_derive(input)?;
    let tokens = [
        input.impl_property(),
        input.impl_permitted_prop_extend(),
        input.make_prop_struct(),
        input.impl_prop_functions(),
        input.impl_definition(),
        input.make_extend(),
    ];

    Ok(tokens.into_iter().collect())
}

struct MacroInput {
    vis: Visibility,
    struct_ident: Ident,
    template_ident: Ident,
    generics: Generics,
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
            struct_ident: orig_struct_name,
            generics: orig_generics @ Generics { where_clause, .. },
            template_ident,
            ..
        } = self;

        let field_names = self.fields.iter().map(|x| &x.ident);
        let new_field_generics: Vec<&Ident> = self.fields.iter().map(|x| &x.prop_type).collect();

        let orig_generics_unbracketed = UnbracketedGenerics(orig_generics);
        let (_, orig_type_g, _) = orig_generics.split_for_impl();

        quote! {
            #[doc(hidden)]
            #vis struct #template_ident<#orig_generics_unbracketed #(#new_field_generics,)*>
            #where_clause
            {
                __irisia_phantom: #PHANTOM_DATA<#orig_struct_name #orig_type_g>,
                #(#field_names: #new_field_generics,)*
            }
        }
    }

    fn impl_property(&self) -> TokenStream {
        let Self {
            struct_ident: orig_struct_name,
            generics,
            ..
        } = self;
        let (impl_g, type_g, where_clause) = generics.split_for_impl();
        let empty = self.impl_property_empty();

        quote! {
            impl #impl_g #PATH_PROPERTY::Property for #orig_struct_name #type_g
            #where_clause {
                #empty
            }
        }
    }

    fn impl_permitted_prop_extend(&self) -> TokenStream {
        let Self {
            struct_ident,
            template_ident,
            ..
        } = self;
        let (_, orig_type_g, _) = split_for_impl_unbracketed(&self.generics);
        let (impl_g, type_g, where_bounds) = self.split_for_impl_template();

        quote! {
            impl<#impl_g> #PATH_PROPERTY::PermittedPropExtend<#template_ident<#type_g>>
                for #struct_ident<#orig_type_g>
            where
                #where_bounds
            {}
        }
    }

    fn impl_property_empty(&self) -> TokenStream {
        let Self { template_ident, .. } = self;

        let (_, type_g, _) = split_for_impl_unbracketed(&self.generics);
        let field_types = self.fields.iter().map(|f| &f.ty);
        let field_init = self.fields.iter().map(|f| {
            let FieldInput { ident, ty, .. } = f;
            quote! {
                #ident: #MACRO_UTILS::get_empty::<#ty>(),
            }
        });

        let path_shortcut = MACRO_UTILS;
        quote! {
            type EmptyProp = #template_ident<
                #type_g
                #(#path_shortcut::EmptyOf<#field_types>,)*
            >;

            const __IRISIA_EMPTY_PROP: Self::EmptyProp = Self::EmptyProp {
                __irisia_phantom: #PHANTOM_DATA,
                #(#field_init)*
            };
        }
    }

    fn impl_prop_functions(&self) -> TokenStream {
        let (impl_g, type_g, where_bounds) = self.split_for_impl_template();
        let Self { template_ident, .. } = self;
        let functions = (0..self.fields.len()).map(|i| self.make_prop_function(i));

        quote! {
            impl<#impl_g> #template_ident<#type_g>
            where
                #where_bounds
            {
                #(#functions)*
            }
        }
    }

    fn make_prop_function(&self, index: usize) -> TokenStream {
        let Self {
            generics,
            template_ident: prop_struct_ident,
            fields: all_fields,
            ..
        } = self;

        let FieldInput {
            ident,
            rename,
            ty: field_type,
            ..
        } = &all_fields[index];

        let (_, generics, _) = split_for_impl_unbracketed(generics);
        let function_name = rename.as_ref().unwrap_or(ident);

        let return_prop_types =
            all_fields
                .iter()
                .enumerate()
                .map(|(i, FieldInput { prop_type, .. })| {
                    if i == index {
                        quote! {__IrisiaValue}
                    } else {
                        quote! {#prop_type}
                    }
                });

        let init_other_prop_fields = all_fields
            .iter()
            .enumerate()
            .filter(|&(i, _)| i != index)
            .map(|(_, f)| {
                let FieldInput { ident, .. } = f;
                quote! {
                    #ident: this.#ident,
                }
            });

        quote! {
            pub fn #function_name<__IrisiaValue>(
                &self,
                value: __IrisiaValue
            ) -> #PATH_PROPERTY::ExtendHelper<
                Self,
                __IrisiaValue,
                #prop_struct_ident<#generics #(#return_prop_types,)*>
            >
            where
                __IrisiaValue: #PATH_PRIVATE::Definition,
                #field_type: #PATH_PROPERTY::PropAssign<__IrisiaValue>,
            {
                #PATH_PROPERTY::ExtendHelper::new(value, |this, value| {
                    #prop_struct_ident {
                        __irisia_phantom: #PHANTOM_DATA,
                        #ident: value,
                        #(#init_other_prop_fields)*
                    }
                })
            }
        }
    }

    fn impl_definition(&self) -> TokenStream {
        let Self {
            template_ident,
            struct_ident,
            ..
        } = self;

        let (_, orig_type_g, _) = split_for_impl_unbracketed(&self.generics);
        let (impl_g, type_g, where_bounds) = self.split_for_impl_template();

        let field_where_bounds = self.fields.iter().map(|f| {
            let FieldInput { ty, prop_type, .. } = f;
            quote! {
                #prop_type: #PATH_PRIVATE::Definition,
                #ty: #PATH_PROPERTY::PropAssign<#prop_type>,
            }
        });

        let init_final_value_field = self.fields.iter().map(|f| {
            let FieldInput { ident, ty, .. } = f;
            quote! {
                #ident: <#ty as #PATH_PROPERTY::PropAssign<_>>::prop_assign(#ident.1),
            }
        });

        let field_generics = self.field_generics();
        let field_names: Vec<&Ident> = self.fields.iter().map(|f| &f.ident).collect();

        quote! {
            impl<#impl_g> #PATH_PRIVATE::Definition for #template_ident<#type_g>
            where
                Self: 'static,
                #where_bounds
                #(#field_where_bounds)*
            {
                type Value = #struct_ident<#orig_type_g>;
                type Storage = #template_ident<
                    #orig_type_g
                    #(#field_generics::Storage,)*
                >;

                fn create(&self) -> (Self::Storage, Self::Value) {
                    let (
                        #(#field_names,)*
                    ) = (
                        #(self.#field_names.create(),)*
                    );

                    (
                        #template_ident {
                            __irisia_phantom: #PHANTOM_DATA,
                            #(#field_names: #field_names.0,)*
                        },
                        #struct_ident {
                            #(#init_final_value_field)*
                        }
                    )
                }

                fn update(&self, storage: &mut Self::Storage) {
                    #(
                        self.#field_names.update(&mut storage.#field_names);
                    )*
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
                prop_type: field_generic,
                ..
            },
        ) = match self.extend_field_index {
            Some(index) => (index, &self.fields[index]),
            None => return TokenStream::new(),
        };

        let Self {
            template_ident,
            struct_ident,
            ..
        } = self;

        let (impl_g, type_g, where_bounds) = self.split_for_impl_template();

        let output_type_args = self.fields.iter().enumerate().map(|(i, g)| {
            if i == extend_field_index {
                let generic = &g.prop_type;
                quote! {
                    #MACRO_UTILS::PropExtendResult<
                        #generic,
                        __IrisiaChild,
                        __IrisiaExt,
                    >
                }
            } else {
                g.prop_type.to_token_stream()
            }
        });

        let other_field_idents = self.fields.iter().enumerate().filter_map(|(i, f)| {
            if i == extend_field_index {
                None
            } else {
                Some(&f.ident)
            }
        });

        let (orig_impl_g, orig_type_g, _) = split_for_impl_unbracketed(&self.generics);

        quote! {
            impl<#impl_g> ::core::ops::Deref for #template_ident<#type_g>
            where
                #where_bounds
            {
                type Target = #field_generic;
                fn deref(&self) -> &Self::Target {
                    &self.#field_ident
                }
            }

            impl<__IrisiaChild, #impl_g> #PATH_PROPERTY::PropExtend<__IrisiaChild>
                for #template_ident<#type_g>
            where
                #where_bounds
                #field_generic: #PATH_PROPERTY::PropExtend<__IrisiaChild>,
                #field_type: #PATH_PROPERTY::PermittedPropExtend<__IrisiaChild>,
            {
                type Output<__IrisiaExt> = #template_ident<#orig_type_g #(#output_type_args,)*>;
                fn prop_extend<__IrisiaExt>(
                    self,
                    f: impl ::core::ops::FnOnce(__IrisiaChild) -> __IrisiaExt
                ) -> Self::Output<__IrisiaExt>
                {
                    #template_ident {
                        __irisia_phantom: #PHANTOM_DATA,
                        #field_ident: #PATH_PROPERTY::PropExtend::prop_extend(self.#field_ident, f),
                        #(#other_field_idents: self.#other_field_idents,)*
                    }
                }
            }

            impl<__IrisiaChild, #orig_impl_g> #PATH_PROPERTY::PermittedPropExtend<__IrisiaChild>
                for #struct_ident<#orig_type_g>
            where
                #where_bounds
                #field_type: #PATH_PROPERTY::PermittedPropExtend<__IrisiaChild>,
            {}
        }
    }

    fn field_generics(&self) -> impl Iterator<Item = &Ident> {
        self.fields.iter().map(|f| &f.prop_type)
    }

    fn split_for_impl_template(&self) -> (TokenStream, TokenStream, WhereClausePredicates<'_>) {
        let (impl_g, type_g, where_bounds) = split_for_impl_unbracketed(&self.generics);
        let field_generics = self.field_generics();
        let field_generics2 = self.field_generics();

        (
            quote! { #impl_g #(#field_generics,)* },
            quote! { #type_g #(#field_generics2,)* },
            where_bounds,
        )
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
