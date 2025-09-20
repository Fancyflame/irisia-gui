use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
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
        input.impl_template_owned_by(),
        input.make_prop_struct(),
        input.make_prop_agent(),
        input.impl_prop_functions(),
        // input.impl_prop_update(),
        input.impl_prop_cast(),
        input.make_extend(),
    ];

    Ok(tokens.into_iter().collect())
}

struct MacroInput {
    vis: Visibility,
    struct_ident: Ident,
    template_ident: Ident,
    agent_ident: Ident,
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
            agent_ident,
            struct_ident: orig_struct_name,
            generics,
            ..
        } = self;
        let (impl_g, type_g, where_clause) = generics.split_for_impl();

        quote! {
            impl #impl_g #PATH_PROPERTY::Property for #orig_struct_name #type_g
            #where_clause {
                type Agent = #agent_ident #type_g;
                // const __IRISIA_PROP_AGENT: &Self::Agent = &#agent_ident(#PHANTOM_DATA);
                fn __irisia_prop_agent<'a>() -> &'a Self::Agent {
                    &#agent_ident(#PHANTOM_DATA)
                }
            }
        }
    }

    fn impl_template_owned_by(&self) -> TokenStream {
        let Self {
            struct_ident,
            template_ident,
            ..
        } = self;
        let (_, orig_type_g, _) = split_for_impl_unbracketed(&self.generics);
        let (impl_g, type_g, where_bounds) = self.split_for_impl_template();

        quote! {
            impl<#impl_g> #PATH_PROPERTY::PropOwnedBy for #template_ident<#type_g>
            where
                #where_bounds
            {
                type Owner = #struct_ident<#orig_type_g>;
            }
        }
    }

    fn make_prop_agent(&self) -> TokenStream {
        let Self {
            vis,
            struct_ident: orig_struct_name,
            generics,
            agent_ident,
            ..
        } = self;

        let (impl_g, type_g, where_clause) = generics.split_for_impl();
        // let functions = (0..self.fields.len()).map(|i| self.make_prop_function(i));
        let empty_part = self.make_agent_empty();

        quote! {
            #[doc(hidden)]
            #vis struct #agent_ident #generics(#PHANTOM_DATA<#orig_struct_name #type_g>);

            impl #impl_g #PATH_PROPERTY::PropertyAgent for #agent_ident #type_g
            #where_clause
            {
                type CastTarget = #orig_struct_name #type_g;

                #empty_part
            }
        }
    }

    fn make_agent_empty(&self) -> TokenStream {
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
            type Empty = #template_ident<
                #type_g
                #(#path_shortcut::EmptyOf<#field_types>,)*
            >;

            fn get_empty(&self) -> Self::Empty {
                Self::Empty {
                    __irisia_phantom: #PHANTOM_DATA,
                    #(#field_init)*
                }
            }
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

    fn impl_prop_update(&self) -> TokenStream {
        let Self {
            agent_ident,
            generics: orig_generics,
            template_ident,
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
                    #MACRO_UTILS::PropUpdateResult<#ty, #s, #o>,
                }
            });

        let field_inits = fields.iter().map(|FieldInput { ident, ty, .. }| {
            quote! {
                #ident: #MACRO_UTILS::prop_update::<#ty, _, _>(
                    this.#ident,
                    other.#ident,
                ),
            }
        });

        let field_types = fields.iter().map(|f| &f.ty);

        let path_shortcut = MACRO_UTILS;
        let path_property = PATH_PROPERTY;
        quote! {
            impl <#orig_impl_g #(#self_generics,)* #(#other_generics,)*>
                #PATH_PROPERTY::PropUpdate<
                    #template_ident<#orig_type_g #(#self_generics,)*>,
                    #template_ident<#orig_type_g #(#other_generics,)*>,
                > for #agent_ident<#orig_type_g>
            where
                #orig_where_bounds
                #(
                    #field_types: #path_property::Property<
                        Agent: #path_property::PropUpdate<
                            #self_generics,
                            #other_generics,
                        >,
                    >,
                )*
            {
                type Output = #template_ident<
                    #orig_type_g
                    #(#output_generics)*
                >;

                fn prop_update(
                    &self,
                    this: #template_ident<#orig_type_g #(#self_generics,)*>,
                    other: #template_ident<#orig_type_g #(#other_generics,)*>
                ) -> Self::Output {
                    #template_ident {
                        __irisia_phantom: #PHANTOM_DATA,
                        #(#field_inits)*
                    }
                }
            }
        }
    }

    fn impl_prop_cast(&self) -> TokenStream {
        let Self {
            agent_ident,
            template_ident,
            ..
        } = self;

        let (impl_g, type_g, where_bounds) = split_for_impl_unbracketed(&self.generics);
        let field_generics: Vec<&Ident> = self.fields.iter().map(|f| &f.prop_type).collect();

        let field_where_bounds = self.fields.iter().map(|f| {
            let FieldInput { ty, prop_type, .. } = f;
            quote! {
                #ty: #PATH_PROPERTY::Property<
                    Agent: #PATH_PROPERTY::PropCast<#prop_type>,
                >,
            }
        });

        let final_struct_init_field = self.fields.iter().map(|f| {
            let ident = &f.ident;
            quote! {
                #ident: #MACRO_UTILS::prop_cast(value.#ident),
            }
        });

        let value_prop = quote! {
            #template_ident<#type_g #(#field_generics,)*>
        };

        quote! {
            impl<#impl_g #(#field_generics,)*> #PATH_PROPERTY::PropCast<#value_prop>
                for #agent_ident<#type_g>
            where
                #where_bounds
                #(#field_where_bounds)*
            {
                fn prop_cast(&self, value: #value_prop) -> Self::CastTarget {
                    Self::CastTarget {
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
                prop_type: field_generic,
                ..
            },
        ) = match self.extend_field_index {
            Some(index) => (index, &self.fields[index]),
            None => return TokenStream::new(),
        };

        let Self { template_ident, .. } = self;

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

        let (_, orig_type_g, _) = split_for_impl_unbracketed(&self.generics);

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
                __IrisiaChild: #PATH_PROPERTY::PropOwnedBy<Owner = #field_type>,
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
        }
    }

    fn split_for_impl_template(&self) -> (TokenStream, TokenStream, WhereClausePredicates<'_>) {
        let (impl_g, type_g, where_bounds) = split_for_impl_unbracketed(&self.generics);
        let field_generics = self.fields.iter().map(|f| &f.prop_type);
        let field_generics2 = field_generics.clone();

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
