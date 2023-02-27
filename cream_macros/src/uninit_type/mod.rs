use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Nothing, Parse, ParseStream},
    punctuated::Punctuated,
    Ident, Token, Visibility,
};

pub mod set;

pub struct UninitTypeList(pub Vec<UninitType>);

impl Parse for UninitTypeList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(UninitTypeList(
            Punctuated::<UninitType, Nothing>::parse_terminated(input)?
                .into_iter()
                .collect(),
        ))
    }
}

pub struct UninitType {
    vis: Visibility,
    type_token: Token!(type),
    ident: Ident,
    semi: Token!(;),
}

impl Parse for UninitType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(UninitType {
            vis: input.parse()?,
            type_token: input.parse()?,
            ident: input.parse()?,
            semi: input.parse()?,
        })
    }
}

impl UninitType {
    pub fn to_output(&self) -> TokenStream {
        let UninitType {
            vis,
            type_token,
            ident,
            semi,
        } = self;

        let module = to_module_name(ident);

        quote! {
            #[cfg(doc)]
            #vis #type_token #ident = FakeNodeCache #semi

            #[cfg(not(doc))]
            #vis #type_token #ident = <#module::__HelperStruct as #module::__HelperTrait>::Output #semi

            #[allow(non_snake_case)]
            #[doc(hidden)]
            #vis mod #module{
                pub struct __HelperStruct;
                pub trait __HelperTrait{
                    type Output;
                }
            }
        }
        .into()
    }
}

fn to_module_name(id: &Ident) -> Ident {
    format_ident!("__module_{id}")
}
