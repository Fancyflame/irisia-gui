use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::Expr;

use crate::component::{Component, FieldAssignMethod, FieldAssignment, ForStmt, IfStmt, Stmt};

use super::BuildMacro;

const_quote! {
    const PATH_CONTROL_FLOW = { irisia::model::control_flow };
    const PATH_COMPONENT = { irisia::model::component };
    const PATH_RC = { ::std::rc::Rc };
}

impl BuildMacro {
    pub fn gen_code(&self) -> TokenStream {
        gen_rc_chained(&self.0)
    }
}

impl Stmt {
    pub fn gen_code(&self) -> TokenStream {
        match self {
            Stmt::Block(block) => gen_chained(&block.stmts),
            Stmt::For(for_stmt) => gen_for(for_stmt),
            Stmt::If(if_stmt) => gen_if(if_stmt),
            Stmt::Component(comp) => gen_component(comp),
            _ => todo!(),
        }
    }
}

fn gen_rc_chained(stmts: &[Stmt]) -> TokenStream {
    let chained = gen_chained(stmts);
    quote! {
        #PATH_RC::new(#chained)
    }
}

fn gen_chained(stmts: &[Stmt]) -> TokenStream {
    match stmts {
        [] => quote! {()},
        [one] => one.gen_code(),
        _ => {
            let (p1, p2) = stmts.split_at(stmts.len() / 2);
            let p1 = gen_chained(p1);
            let p2 = gen_chained(p2);
            quote! {
                (#p1, #p2)
            }
        }
    }
}

fn gen_for(
    ForStmt {
        pattern,
        expr,
        body,
        get_key,
    }: &ForStmt,
) -> TokenStream {
    let body = gen_chained(&body.stmts);
    quote! {
        ::std::vec::Vec::from_iter(
            ::core::iter::Iterator::map(
                ::core::iter::IntoIterator::into_iter(#expr),
                |#pattern| (#get_key, #body)
            )
        )
    }
}

fn gen_if(
    IfStmt {
        condition,
        then_branch,
        else_branch,
    }: &IfStmt,
) -> TokenStream {
    let then_branch = gen_chained(&then_branch.stmts);
    let else_branch = match else_branch {
        Some(b) => b.gen_code(),
        None => quote! {()},
    };

    quote! {
        if #condition {
            #PATH_CONTROL_FLOW::branch::branch_a(
                #then_branch
            )
        } else {
            #PATH_CONTROL_FLOW::branch::branch_b(
                #else_branch
            )
        }
    }
}

fn gen_component(
    Component {
        type_path,
        fields,
        body,
    }: &Component,
) -> TokenStream {
    let mut fields: Vec<&FieldAssignment> = fields.iter().collect();
    let mut _body_fa = None;
    if !body.is_empty() {
        fields.push(_body_fa.insert(FieldAssignment {
            name: format_ident!("children"),
            method: FieldAssignMethod::HostingSignal,
            value: Expr::Verbatim(gen_rc_chained(&body)),
        }));
    };

    let defs = binary_fold(&fields, &|fa| {
        let FieldAssignment {
            name: _,
            value,
            method,
        } = fa;

        match method {
            FieldAssignMethod::HostingSignal => {
                quote! {
                    #PATH_COMPONENT::proxy_signal_helper::check_eq(#value).get()
                }
            }
            FieldAssignMethod::Direct => {
                quote! {
                    #PATH_COMPONENT::definition::DirectAssign(#value)
                }
            }
        }
    });

    let names_tuple = binary_fold(&fields, &|fa| fa.name.to_token_stream());
    let assignments = fields.iter().map(|fa| {
        let name = &fa.name;
        let value = match fa.method {
            FieldAssignMethod::HostingSignal => {
                quote! {
                    irisia::coerce_hook!(#name)
                }
            }
            FieldAssignMethod::Direct => {
                quote! { #name }
            }
        };

        quote! {
            #name: ::core::option::Option::Some(#value),
        }
    });

    let create_fn = quote! {
        |#names_tuple| {
            #type_path {
                #(#assignments)*
                ..::core::default::Default::default()
            }
        }
    };

    quote! {
        {
            #PATH_COMPONENT::UseComponent::<#type_path, _, _>::new(
                #create_fn,
                #defs,
            )
        }
    }
}

fn binary_fold<F>(slice: &[&FieldAssignment], for_each: &F) -> TokenStream
where
    F: Fn(&FieldAssignment) -> TokenStream,
{
    match slice {
        [] => quote! {()},
        [one] => for_each(one),
        _ => {
            let (a, b) = slice.split_at(slice.len() / 2);
            let a = binary_fold(a, for_each);
            let b = binary_fold(b, for_each);
            quote! {(#a, #b)}
        }
    }
}
