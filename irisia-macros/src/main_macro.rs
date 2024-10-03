use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, Parser},
    parse_quote, Error, ItemFn, Result,
};

fn handle(mut item: ItemFn) -> Result<TokenStream2> {
    if item.sig.asyncness.take().is_none() {
        return Err(Error::new_spanned(
            &item.sig,
            "function is expected to be asynchronous",
        ));
    }

    let block = &item.block;

    item.block = parse_quote! {{
        irisia::start_runtime(
            async move #block
        )
    }};

    Ok(item.into_token_stream())
}

pub fn main_macro(input: TokenStream) -> TokenStream {
    let input: TokenStream2 = input.into();

    match ItemFn::parse.parse2(input.clone()).and_then(handle) {
        Ok(result) => result,
        Err(err) => {
            let err = err.to_compile_error();
            quote! {
                #err
                #input
            }
        }
    }
    .into()
}
