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
            rename,
            style,
            event_dispatcher,
            event_emitting_key,
            children,
        } = self;

        let props = gen_props(element, props);

        if rename.is_some() {
            quote!(::std::compile_error("rename is not support yet")).to_tokens(tokens);
            return;
        }

        let event_listeners = gen_event_listeners(
            event_dispatcher.as_ref().map(|x| x.as_ref()),
            event_emitting_key.as_ref(),
        );

        let children = {
            let mut tokens = TokenStream::new();
            stmts_to_tokens(&mut tokens, &children);
            tokens
        };

        quote! {
            cream::structure::add_child::<#element, _, _>(
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
        cream::__TypeHelper::<
            <#element as cream::element::Element>::Props<'_>
        > {
            #(#idents: #exprs,)*
            ..::std::default::Default::default()
        }
    }
}

fn gen_event_listeners(
    event_dispatcher: Option<&Expr>,
    event_emitting_key: Option<&Expr>,
) -> TokenStream {
    match (event_dispatcher, event_emitting_key) {
        (_, None) => quote!(cream::event::EventEmitter::new_empty()),

        (Some(disp), Some(key)) => quote!((#disp).emitter(#key)),

        (None, Some(_)) => {
            quote!(::std::compile_error!(
                "the event emitter cannot be sent to the target element, \
                because there is no event dispatcher provided"
            ))
        }
    }
}
