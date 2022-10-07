use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, Type, Visibility};

use crate::{generate::build_tree, parse::Component};

// 键名: (类型, 是否是重命名)
pub type FieldMap = HashMap<Ident, (Option<Visibility>, Type)>;

pub fn generate(comp: Component) -> TokenStream {
    let Component {
        visibility,
        name,
        data,
        fields,
        computed,
        methods,
        watch,
        body,
    } = comp;
    let mut field_map = FieldMap::new();

    build_tree::build_tree(body, &mut field_map).unwrap();

    let mut inner = Vec::with_capacity(field_map.len());
    for (k, (vis, ty)) in field_map.into_iter() {
        let item = if let Some(vis) = vis {
            quote!(#vis #k: #ty)
        } else {
            quote! {
                #[allow(non_snake_case)]
                #k: #ty
            }
        };
        inner.push(item);
    }

    let expr = quote! {
        #visibility struct #name{
            #(#inner),*
        }
    };
    println!("{}", expr.to_string());

    expr.into()
}
