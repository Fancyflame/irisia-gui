use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Expr, Ident, Type};

use super::ElementStmt;

impl ToTokens for ElementStmt {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ElementStmt {
            element,
            props,
            rename,
            style,
            event_listeners,
            children,
        } = self;

        let props = gen_props(element, props);

        quote! {
            ::cream_core::structure::add_child::<#element, _, _, _, _, _>(
                #props,
                #style,

            )
        }
        .to_tokens(tokens)
    }
}

fn gen_props(element: &Type, props: &[(Ident, Expr)]) -> TokenStream {
    let idents = props.iter().map(|x| &x.0);
    let exprs = props.iter().map(|x| &x.1);
    quote! {
        ::cream_core::__macro_helper::__TypeHelper::<
            <#element as ::cream_core::element::Element>::Props
        > {
            #(#idents: #exprs,)*
            ..::std::default::Default::default()
        }
    }
}
