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

        let children = {
            let mut tokens = TokenStream::new();
            stmts_to_tokens(&mut tokens, children);
            tokens
        };

        let oncreate = oncreate.clone().unwrap_or_else(|| parse_quote!(|_| {}));

        quote! {
            irisia::structure::add_child::<#element, _, _, _>(
                #props,
                #style,
                #oncreate,
                #children
            )
        }
        .to_tokens(tokens)
    }
}

fn gen_props(element: &Type, props: &[(Ident, Expr)]) -> TokenStream {
    let idents = props.iter().map(|x| &x.0);
    let exprs = props.iter().map(|x| &x.1);
    quote! {
        irisia::__TypeHelper::<
            <#element as irisia::element::Element>::Props<'_>
        > {
            #(#idents: #exprs,)*
            ..::std::default::Default::default()
        }
    }
}
