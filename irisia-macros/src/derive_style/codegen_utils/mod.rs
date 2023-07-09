use std::rc::Rc;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::Generics;

mod append_path;
mod impl_default;

#[derive(Clone)]
pub struct CodeGenerator {
    tokens: TokenStream,
    ident: Ident,
    variant: Rc<TokenStream>,
    generics: Generics,
}

impl CodeGenerator {
    pub fn new(ident: Ident, variant: Option<&Ident>, generics: Generics) -> Self {
        Self {
            tokens: TokenStream::new(),
            ident,
            variant: Rc::new(match variant {
                Some(v) => quote!(Self::#v),
                None => quote!(Self),
            }),
            generics,
        }
    }

    pub fn impl_style(&mut self) {
        self.impl_trait(quote!(irisia::Style), quote!());
    }

    pub fn impl_trait<T, U>(&mut self, trait_tokens: T, body: U)
    where
        T: ToTokens,
        U: ToTokens,
    {
        let (impl_gen, type_gen, where_clause) = self.generics.split_for_impl();
        let ident = &self.ident;
        quote! {
            #[automatically_derived]
            impl #impl_gen #trait_tokens for #ident #type_gen
            #where_clause
            {
                #body
            }
        }
        .to_tokens(&mut self.tokens);
    }

    pub fn append_fn<T>(&mut self, body: T)
    where
        T: ToTokens,
    {
        let (impl_gen, type_gen, where_clause) = self.generics.split_for_impl();
        let ident = &self.ident;
        quote! {
            impl #impl_gen #ident #type_gen
            #where_clause
            {
                #body
            }
        }
        .to_tokens(&mut self.tokens);
    }

    pub fn finish(self) -> TokenStream {
        self.tokens
    }
}
