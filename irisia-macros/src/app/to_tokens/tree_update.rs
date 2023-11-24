use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Expr, Ident, Stmt};

use crate::app::parse::ElementStmt;

#[derive(Clone)]
enum Seg {
    ChainFormer,
    ChainLatter,
    RepeatData,
    RepeatTree,
    BranchIndex,
    BranchTree(u32),
    NodeProp(Ident),
    NodeStyle,
    NodeSlot,
}

fn build_stmt<F>(mut expr: TokenStream, path: &[Seg], assigner: F) -> TokenStream
where
    F: FnOnce(TokenStream) -> TokenStream,
{
    let mut path_iter = path.into_iter();
    fn repeat_var() -> TokenStream {
        quote!(__irisia_repeat_var)
    }

    for seg in &mut path_iter {
        match seg {
            Seg::ChainFormer => expr = quote!(#expr.former),
            Seg::ChainLatter => expr = quote!(#expr.latter),
            Seg::RepeatTree => {
                let rv = repeat_var();
                let update_child = build_stmt(rv.clone(), path_iter.as_slice(), assigner);
                return quote! {
                    #expr.update_tree(|#rv| {
                        #update_child
                    })
                };
            }
            Seg::RepeatData => {
                let rv = repeat_var();
                let update_child = build_stmt(rv.clone(), path_iter.as_slice(), assigner);
                return build_stmt(quote! {}, path_iter.as_slice(), assigner);
            }
        }
    }

    assigner(expr)
}

fn track_el(watcher_list: &mut Vec<Stmt>, path: &Vec<Seg>, el: &ElementStmt) {}
