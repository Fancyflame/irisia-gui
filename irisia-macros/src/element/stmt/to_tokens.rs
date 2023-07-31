use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
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

        let children = {
            let mut tokens = TokenStream::new();
            stmts_to_tokens(&mut tokens, children);
            tokens
        };

        let oncreate = oncreate.clone().unwrap_or_else(|| parse_quote!(|_| {}));

        quote! {
            irisia::structure::add_child::<#element, _, _, _, _>(
                #props,
                #style,
                #children,
                #oncreate,
            )
        }
        .to_tokens(tokens)
    }
}

fn gen_props(element: &Type, props: &[(Ident, Expr)]) -> TokenStream {
    let set_prop = props.iter().map(|x| {
        let id = &x.0;
        let mut set_id = format_ident!("set_{}", id);
        set_id.set_span(id.span());
        set_id
    });
    let exprs = props.iter().map(|x| &x.1);

    quote! {
        <<#element as irisia::element::Element>::BlankProps as ::std::default::Default>::default()
            #(.#set_prop((#exprs,)))*
    }
}
