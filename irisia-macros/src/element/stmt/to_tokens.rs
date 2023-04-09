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
            style,
            event_dispatcher,
            identity,
            children,
        } = self;

        let props = gen_props(element, props);

        let event_listeners = gen_event_listeners(
            event_dispatcher.as_ref().map(|x| x.as_ref()),
            identity.as_ref(),
        );

        let children = {
            let mut tokens = TokenStream::new();
            stmts_to_tokens(&mut tokens, children);
            tokens
        };

        quote! {
            irisia::structure::add_child::<#element, _, _, _>(
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
        irisia::__TypeHelper::<
            <#element as irisia::element::Element>::Props<'_>
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
        (_, None) => {
            quote!(irisia::event::event_dispatcher::emitter::CreatedEventEmitter::new_empty())
        }

        (Some(disp), Some(key)) => quote!((#disp).created_event_emitter(#key)),

        (None, Some(_)) => {
            quote!(::std::compile_error!(
                "the event emitter cannot be sent to the target element, \
                because there is no event dispatcher provided"
            ))
        }
    }
}
