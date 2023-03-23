use quote::{quote, ToTokens};
use syn::{parenthesized, parse::Parse, punctuated::Punctuated, token::Paren, Expr, Token};

pub enum ElementCommand {
    Slot(Expr),
    Init(InitCommand),
}

pub struct InitCommand {
    pub args: Punctuated<Expr, Token![,]>,
}

impl ToTokens for ElementCommand {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Slot(ex) => quote! {
                cream_core::structure::ApplySlot::new(#ex)
            }
            .to_tokens(tokens),
            Self::Init(_) => unreachable!(),
        }
    }
}

impl Parse for InitCommand {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if !input.peek(Paren) {
            return Ok(InitCommand {
                args: Punctuated::new(),
            });
        }

        let content;
        parenthesized!(content in input);
        let punct = Punctuated::<Expr, Token![,]>::parse_terminated(&content)?;
        Ok(InitCommand { args: punct })
    }
}
