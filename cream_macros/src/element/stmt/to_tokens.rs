use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Expr, Ident, LitStr, Type};

use crate::expr::state_block::stmts_to_tokens;

use super::ElementStmt;

impl ToTokens for ElementStmt {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ElementStmt {
            element,
            props,
            rename,
            style,
            event_setter,
            event_listener_channel,
            children,
        } = self;

        let props = gen_props(element, props);

        if rename.is_some() {
            quote!(::std::compile_error("rename is not support yet")).to_tokens(tokens);
            return;
        }

        let event_listeners = gen_event_listeners(
            event_setter.as_ref().map(|x| x.as_ref()),
            event_listener_channel,
        );

        let children = {
            let mut tokens = TokenStream::new();
            stmts_to_tokens(&mut tokens, &children);
            tokens
        };

        quote! {
            cream_core::structure::add_child::<#element, _, _>(
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
        cream_core::__TypeHelper::<
            <#element as cream_core::element::Element>::Props<'_>
        > {
            #(#idents: #exprs,)*
            ..::std::default::Default::default()
        }
    }
}

fn gen_event_listeners(
    event_setter: Option<&Expr>,
    listener: &Option<(LitStr, Option<Expr>)>,
) -> TokenStream {
    match (event_setter, listener) {
        (_, None) => quote!(cream_core::event::EventEmitter::new_empty()),

        (Some(event_setter), Some((name, maybe_with_key))) => match maybe_with_key {
            None => quote!((#event_setter).to_emitter(#name)),
            Some(with_key) => quote!((#event_setter).to_emitter_with_key(#name, #with_key)),
        },

        (None, Some((name, _))) => {
            let string = format!(
                "channel `{}` cannot be mount, because no event setter provided",
                name.value()
            );
            quote!(::std::compile_error!(#string))
        }
    }
}
