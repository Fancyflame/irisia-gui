use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{parse_quote, Error, ItemFn, Result};

pub fn main_macro(mut item: ItemFn) -> Result<TokenStream> {
    if item.sig.asyncness.take().is_none() {
        return Err(Error::new_spanned(
            &item.sig,
            "function is expected to be asynchronous",
        ));
    }

    let block = &item.block;

    item.block = parse_quote! {{
        cream::start_runtime(
            async move #block
        )
    }};

    Ok(item.into_token_stream())
}
