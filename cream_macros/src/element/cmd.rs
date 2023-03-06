use quote::{quote, ToTokens};
use syn::Expr;

pub enum ElementCommand {
    Slot(Expr),
    InitRender {
        event_src: Expr,
        cache_box: Expr,
        render_content: Expr,
    },
    InitBuild {
        event_src: Expr,
    },
}

impl ToTokens for ElementCommand {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Slot(ex) => quote! {
                ::cream_core::structure::ApplySlot::new(#ex)
            }
            .to_tokens(tokens),
            Self::InitRender { .. } | Self::InitBuild { .. } => unreachable!(),
        }
    }
}
