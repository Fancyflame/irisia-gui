use proc_macro2::TokenStream;
use syn::{ItemStruct, Result};

// attr详情<https://doc.rust-lang.org/stable/book/ch19-06-macros.html#attribute-like-macros>
pub fn props(_attr: TokenStream, _item: ItemStruct) -> Result<TokenStream> {
    todo!();
}
