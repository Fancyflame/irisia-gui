use quote::{quote, ToTokens};
use syn::Expr;

pub enum Command {
    Slot(Expr),
}

impl ToTokens for Command {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Command::Slot(ex) => quote! {
                ::cream_core::structure::ApplySlot::new(#ex)
            },
        }
        .to_tokens(tokens)
    }
}
