use std::{collections::HashMap, fmt::Debug};

use syn::{Expr, Ident};

use super::node::{Children, Node, StyleNode};

#[inline]
fn ts<T: quote::ToTokens>(t: &T) -> String {
    t.to_token_stream().to_string()
}

impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(debug_assertions)]
        {
            let Node {
                span: _,
                node_type,
                expr_queue,
                required_args,
                optional_args,
                id_rename,
                style,
                auto_parent,
                children,
            } = self;

            let node_type = ts(node_type);

            let mut displays = Vec::new();

            let mut cmd_block = Vec::new();
            {
                if let Some(id_rename) = id_rename {
                    cmd_block.push(format!("[id_rename] {}", id_rename));
                }

                if let Some(par) = style {
                    cmd_block.push(format!("[auto_parent] {:?}", par));
                }
            }
            if !cmd_block.is_empty() {
                displays.push(cmd_block);
            }

            let a = display_arg(expr_queue, required_args, optional_args);
            if !a.is_empty() {
                displays.push(a);
            }

            match children {
                Children::None => {}
                Children::Single(node) => displays.push(vec![format!("{:?}", node)]),
                Children::Multiple(vec) => {
                    let mut dst = Vec::new();
                    let (first, rest) = vec.split_first().unwrap();
                    dst.push(format!("{:?}", first));
                    dst.extend(rest.iter().map(|x| format!("\n{:?}", x)));

                    displays.push(dst);
                }
            };

            let mut string = String::new();

            let len = displays.len();
            displays.into_iter().enumerate().for_each(|(index, x)| {
                string += &x.join(",\n");
                if index != len - 1 {
                    string += ",\n\n";
                }
            });

            let string = string.replace("\n", "\n    ");
            let debug = format!(
                "\
{node_type} {{
    {string}
}}",
            );

            write!(f, "{}", debug)
        }
        #[cfg(not(debug_assertions))]
        write!(f, "{{ Debug information is only for debug mode }}")
        //Ok(())
    }
}

impl Debug for StyleNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(debug_assertions)]
        {
            let StyleNode {
                span: _,
                expr_queue,
                required_args,
                optional_args,
            } = self;

            let a = display_arg(expr_queue, required_args, optional_args)
                .join(",\n")
                .replace("\n", "\n    ");

            let debug = format!(
                "\
{{
    {}
}}",
                a
            );

            write!(f, "{}", debug)
        }
        #[cfg(not(debug_assertions))]
        write!(f, "{{ Debug information is only for debug mode }}")
    }
}

fn display_arg(
    expr_queue: &Vec<Expr>,
    required_args: &Vec<usize>,
    optional_args: &HashMap<Ident, usize>,
) -> Vec<String> {
    required_args
        .iter()
        .enumerate()
        .map(|(i, x)| format!("{i}: {}", ts(&expr_queue[*x])))
        .chain(
            optional_args
                .iter()
                .map(|(k, x)| format!("{k}: {}", ts(&expr_queue[*x]))),
        )
        .collect()
}
