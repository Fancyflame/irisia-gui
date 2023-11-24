use proc_macro2::TokenStream;
use quote::quote;

use crate::{app::parse::ElementStmt, expr::StateExpr};

fn create_tree(stmt: &StateExpr<ElementStmt>) -> TokenStream {
    match stmt {
        StateExpr::Block(block) => {
            let mut block_stmts = block.stmts.iter();
            let mut tokens = match block_stmts.next() {
                Some(s) => create_tree(s),
                None => return quote!(()),
            };

            for s in block_stmts {
                let tree = create_tree(s);
                tokens = quote! {::irisia::structure::Chain::new(#tokens, #tree)};
            }
            tokens
        }

        StateExpr::Command(_) => unreachable!(),

        StateExpr::Conditional(cond) => {}
    }
}

fn create_el(
    ElementStmt {
        element,
        props,
        styles,
        oncreate,
        child,
    }: &ElementStmt,
) -> TokenStream {
    let props_key = props.keys();
    let props_val = props.values();
    let slot = create_el(child);

    quote! {
        ::irisia::dom::RcElementModel::<#element, _>::new(
            <<#element>::BlankProps as ::std::default::Default>::default()
                #(.#props_key(#props_val))*,
            #styles,
            #slot,
            #oncreate,
        )
    }
}
