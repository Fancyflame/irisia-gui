use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::{
    Error, ItemFn, Result,
    parse::{Parse, Parser},
    parse_quote,
};

pub fn main_macro(mut item: ItemFn) -> Result<TokenStream2> {
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
