use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Expr, Ident, Type};

use crate::expr::state_block::stmts_to_tokens;

use super::ElementStmt;

impl ToTokens for ElementStmt {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ElementStmt {
            element,
            props,
            _rename: _,
            style,
            event_src,
            event_listeners,
            children,
        } = self;

        let props = gen_props(element, props);

        let event_listeners = gen_event_listeners(
            event_src
                .as_ref()
                .expect("inner error: event src need to be set"),
            event_listeners,
        );

        let children = {
            let mut tokens = TokenStream::new();
            stmts_to_tokens(&mut tokens, &children);
            tokens
        };

        quote! {
            ::cream_core::structure::add_child::<#element, _, _, _, _>(
                #props,
                #style,
                #event_listeners,
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
        ::cream_core::__TypeHelper::<
            <#element as ::cream_core::element::Element>::Props<'_>
        > {
            #(#idents: #exprs,)*
            ..::std::default::Default::default()
        }
    }
}

fn gen_event_listeners(event_src: &Expr, event_listeners: &[(Type, Expr)]) -> TokenStream {
    let mut output = quote! {
        ::std::clone::Clone::clone(#event_src)
    };

    for (event, value) in event_listeners {
        quote! {
            .listen::<#event, _>(#value)
        }
        .to_tokens(&mut output);
    }

    output
}
