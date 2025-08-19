use darling::{FromDeriveInput, FromField, Result as DarlingResult, ast, util::Flag};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{DeriveInput, Generics, Ident, LitStr};

use crate::{consts::*, pname::pname_inner};

const_quote! {
    const PATH_PROPERTY = {
        #PATH_COMPONENT::property
    };
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(prop), supports(struct_named))]
struct MacroOpts {
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<(), FieldOpts>,
}

#[derive(Debug, FromField)]
#[darling(attributes(prop), and_then = "FieldOpts::validate")]
struct FieldOpts {
    ident: Option<syn::Ident>,
    ty: syn::Type,

    #[darling(default)]
    expand: Option<Vec<LitStr>>,

    #[darling(default)]
    expand_all: Flag,

    #[darling(default)]
    rename: Option<LitStr>,

    #[darling(default)]
    hidden: Flag,
}

impl FieldOpts {
    /// 互斥校验：不能同时声明 expand 与 expand_all
    fn validate(self) -> DarlingResult<Self> {
        if self.expand.is_some() && self.expand_all.is_present() {
            return Err(darling::Error::custom(
                "`prop`: `expand` and `expand_all` cannot be used together",
            ));
        }
        Ok(self)
    }
}

pub fn derive_prop(input: DeriveInput) -> TokenStream {
    let opts = match MacroOpts::from_derive_input(&input) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };

    let field_cfgs: Vec<FieldOpts> = match opts.data {
        ast::Data::Struct(fields) => fields.fields,
        _ => unreachable!(),
    };

    let header_maker = PropHeaderMaker {
        struct_ident: &opts.ident,
        struct_generics: &opts.generics,
    };

    let mut tokens = TokenStream::new();
    for field in &field_cfgs {
        field.to_tokens(header_maker).to_tokens(&mut tokens);
    }

    tokens.into()
}

impl FieldOpts {
    fn to_tokens(&self, header_maker: PropHeaderMaker) -> TokenStream {
        let mut tokens = TokenStream::new();

        self.this_as_prop_to_tokens(header_maker)
            .to_tokens(&mut tokens);

        if let Some(expand_vec) = &self.expand {
            for expand_name_str in expand_vec {
                self.expand_to_tokens(header_maker, expand_name_str)
                    .to_tokens(&mut tokens);
            }
        }

        tokens
    }

    fn this_as_prop_to_tokens(&self, header_maker: PropHeaderMaker) -> TokenStream {
        let Self {
            ident: field_name,
            ty: field_type,
            hidden,
            ..
        } = self;

        if hidden.is_present() {
            return TokenStream::new();
        }

        let prop_name = match self.rename.as_ref() {
            Some(rename) => rename.value(),
            None => field_name.as_ref().unwrap().to_string(),
        };

        let header = header_maker.make(&prop_name, field_type);
        quote! {
            #header {
                type ReturnSelf = Self;
                fn set(mut self, value: #field_type) -> Self::ReturnSelf {
                    self.#field_name = value;
                    self
                }
            }
        }
    }

    fn expand_to_tokens(
        &self,
        header_maker: PropHeaderMaker,
        expand_name_str: &LitStr,
    ) -> TokenStream {
        let Self {
            ident: field_name,
            ty: field_type,
            ..
        } = self;

        let expand_name_str = expand_name_str.value();
        let expand_name_type = pname_inner(&expand_name_str);

        let expand_type = quote! {
            #PATH_PROPERTY::PropertyType<#field_type, #expand_name_type>
        };
        let header = header_maker.make(&expand_name_str, &expand_type);

        quote! {
            #header {
                type ReturnSelf = Self;
                fn set(mut self, value: #expand_type) -> Self::ReturnSelf {
                    self.#field_name = #PATH_PROPERTY::SetProperty::<#expand_name_type, _>::set(
                        self.#field_name,
                        value
                    );
                    self
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct PropHeaderMaker<'a> {
    struct_ident: &'a Ident,
    struct_generics: &'a Generics,
}

impl PropHeaderMaker<'_> {
    fn make(&self, prop_name: &str, prop_type: impl ToTokens) -> TokenStream {
        let prop_name = pname_inner(prop_name);
        let struct_ident = &self.struct_ident;
        let (impl_g, type_g, where_clause) = self.struct_generics.split_for_impl();

        quote! {
            impl #impl_g #PATH_PROPERTY::GetPropertyType<#prop_name>
                for #struct_ident #type_g #where_clause
            {
                type PropertyType = #prop_type;
            }

            impl #impl_g #PATH_PROPERTY::SetProperty<#prop_name, #prop_type>
                for #struct_ident #type_g #where_clause
        }
    }
}
