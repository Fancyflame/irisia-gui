use std::sync::atomic::AtomicU64;

use crate::parse::node::{style::StyleType, Children, Node};

use super::assemble::FieldMap;

use quote::format_ident;
use syn::{parse_quote, spanned::Spanned, Ident, Result, TypePath, VisPublic, Visibility};

pub fn build_tree(root: Node, map: &mut FieldMap) -> Result<()> {
    write_node(root, map, None)
}

fn write_node(node: Node, map: &mut FieldMap, parent_type: Option<&TypePath>) -> Result<()> {
    let node_type = node.node_type;

    {
        // 自动样式必须是最后一个元素
        let mut maybe_last = false;

        for x in node.styles.into_iter() {
            if maybe_last {
                parse_err!(
                    x.style.span(),
                    "Automatic generated style element must be the last one"
                );
            } else {
                let style = match x.style {
                    StyleType::Auto(ti) => {
                        maybe_last = true;
                        match parent_type {
                            Some(pt) => {
                                parse_quote!(<#pt as ::chanpagne::Trait>::Type)
                            }
                            None => parse_err!(
                                ti.span(),
                                "Cannot generate automatic style for this element"
                            ),
                        }
                    }
                    StyleType::Specified(tp) => tp,
                };
            }
        }
    }

    // 插入struct声明表
    let (name, vis) = match node.id_rename {
        Some(id) => (id, Some(parse_quote!(pub))),
        None => (generate_ident(), None),
    };

    // 写入孩子节点
    match node.children {
        Children::None => {}
        Children::Single(child) => write_node(*child, map, Some(&node_type))?,
        Children::Multiple(vec) => {
            for x in vec.into_iter() {
                write_node(x, map, Some(&node_type))?;
            }
        }
    }

    map.insert(name, (vis, syn::Type::Path(node_type)));

    Ok(())
}

// 随便生成一个Id
fn generate_ident() -> Ident {
    lazy_static::lazy_static! {
        static ref COUNTER: AtomicU64 = AtomicU64::new(0);
    }

    format_ident!(
        "__ChampagneGUIAutoGen__No_{}",
        COUNTER.fetch_add(1, std::sync::atomic::Ordering::Release)
    )
}
