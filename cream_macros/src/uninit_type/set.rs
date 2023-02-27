use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Nothing, Parse},
    punctuated::Punctuated,
    Path, Token, Type,
};

use super::to_module_name;

pub struct SetTypeList(pub Vec<SetType>);

impl Parse for SetTypeList {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(SetTypeList(
            Punctuated::<SetType, Nothing>::parse_terminated(input)?
                .into_iter()
                .collect(),
        ))
    }
}

pub struct SetType {
    type_token: Token!(type),
    path: Path,
    equal: Token!(=),
    ty: Type,
    semi: Token!(;),
}

impl Parse for SetType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(SetType {
            type_token: input.parse()?,
            path: input.parse()?,
            equal: input.parse()?,
            ty: input.parse()?,
            semi: input.parse()?,
        })
    }
}

impl SetType {
    pub fn to_output(&self) -> TokenStream {
        let SetType {
            type_token,
            path,
            equal,
            ty,
            semi,
        } = self;

        let mut path = path.clone();

        let module = &mut path
            .segments
            .last_mut()
            .expect("cannot find uninitialized type")
            .ident;

        *module = to_module_name(&module);

        quote! {
            impl #path::__HelperTrait for #path::__HelperStruct {
                #type_token Output #equal #ty #semi
            }
        }
        .into()
    }
}
