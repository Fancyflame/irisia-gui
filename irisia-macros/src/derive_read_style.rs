use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    punctuated::{Pair, Punctuated},
    Data, DeriveInput, Error, Field, Fields, Result, Token,
};

pub fn derive(
    DeriveInput {
        ident,
        generics,
        data,
        ..
    }: DeriveInput,
) -> Result<TokenStream> {
    let fields = match data {
        Data::Struct(s) => s.fields,
        Data::Enum(_) | Data::Union(_) => {
            return Err(Error::new(Span::call_site(), "only struct is support"))
        }
    };

    let inner = match fields {
        Fields::Named(named) => {
            let inner = fields_iter(named.named);
            quote! {{#inner}}
        }
        Fields::Unnamed(unnamed) => {
            let inner = fields_iter(unnamed.unnamed);
            quote! {(#inner)}
        }
        Fields::Unit => quote! {},
    };

    let mut generics = generics;
    add_trait_bounds(&mut generics);

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let output = quote! {
        impl #impl_generics irisia::style::ReadStyle for #ident #ty_generics
        #where_clause
        {
            fn read_style_into(&self, buf: &mut irisia::style::StyleBuffer) {
                #inner
            }
        }
    };

    Ok(output)
}

fn add_trait_bounds(generics: &mut syn::Generics) {
    for param in &mut generics.params {
        if let syn::GenericParam::Type(type_param) = param {
            type_param
                .bounds
                .push(syn::parse_quote!(irisia::style::ReadStyle));
        }
    }
}

fn fields_iter(fields: Punctuated<Field, Token![,]>) -> TokenStream {
    let mut tokens = TokenStream::new();
    for pair in fields.pairs() {
        let (Field { ident, .. }, comma) = match pair {
            Pair::Punctuated(t, p) => (t, Some(p)),
            Pair::End(t) => (t, None),
        };

        quote! {
            irisia::style::ReadStyle::read_style_into(&self.#ident, buf) #comma
        }
        .to_tokens(&mut tokens);
    }
    tokens
}
