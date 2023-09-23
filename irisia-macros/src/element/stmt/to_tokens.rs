use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_quote, Expr, Ident, Type};

use crate::expr::state_block::stmts_to_tokens;

use super::ElementStmt;

impl ToTokens for ElementStmt {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ElementStmt {
            element,
            props,
            style,
            oncreate,
            children,
        } = self;

        let props = gen_props(element, props);
        let style = match style {
            Some(style) => style.into_token_stream(),
            None => quote!(()),
        };
        let children = stmts_to_tokens(children);
        let oncreate = oncreate.clone().unwrap_or_else(|| parse_quote!(|_| {}));

        quote! {
            irisia::structure::Once(
                irisia::element::one_child::<#element, _, _, _, _>(
                    #props,
                    #style,
                    #children,
                    #oncreate,
                )
            )
        }
        .to_tokens(tokens)
    }
}

fn gen_props(element: &Type, pairs: &[(Ident, Expr)]) -> TokenStream {
    let props = pairs.iter().map(|x| &x.0);
    let exprs = pairs.iter().map(|x| &x.1);

    quote! {
        <<#element as irisia::element::Element>::BlankProps as ::std::default::Default>::default()
            #(.#props(#exprs))*
    }
}
