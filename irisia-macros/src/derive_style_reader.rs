use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    punctuated::{Pair, Punctuated},
    Data, DeriveInput, Error, Field, Fields, Result, Token,
};

pub fn derive_style_reader(
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

    let (impl_gen, type_gen, where_clause) = generics.split_for_impl();

    let output = quote! {
        impl #impl_gen irisia::StyleReader for #ident #type_gen
        #where_clause
        {
            fn read_style(_container: impl irisia::style::StyleContainer) -> Self {
                #ident #inner
            }
        }
    };

    Ok(output)
}

fn fields_iter(fields: Punctuated<Field, Token![,]>) -> TokenStream {
    let mut tokens = TokenStream::new();
    for pair in fields.pairs() {
        let (
            Field {
                ident,
                colon_token,
                ty,
                ..
            },
            comma,
        ) = match pair {
            Pair::Punctuated(t, p) => (t, Some(p)),
            Pair::End(t) => (t, None),
        };

        quote! {
            #ident #colon_token <#ty as irisia::StyleReader>::read_style(&_container) #comma
        }
        .to_tokens(&mut tokens);
    }
    tokens
}
