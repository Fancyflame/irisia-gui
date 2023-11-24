use std::collections::HashMap;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, ToTokens};
use syn::{parse_quote, Expr, Type};

use super::{parse::ElementStmt, App};

mod tree_create;
mod tree_update;
mod type_infer;

type FieldMap = HashMap<Ident, Type>;

impl App {
    pub fn to_tokens(&self) -> TokenStream {}

    fn field_watchers(&self) -> Vec<Expr> {
        let mut watcher_list = Vec::new();
        if let Some(child_nodes) = &self.child_node {
            track_el(child_nodes, &mut watcher_list);
        };
        watcher_list
    }
}
