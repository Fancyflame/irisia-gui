use std::collections::VecDeque;

use proc_macro2::TokenStream;
use quote::quote;

use crate::component::{
    codegen::{PATH_CONTROL_FLOW, VAR_INPUT_DP},
    ForStmt, IfStmt, Stmt, UseSlot,
};

impl Stmt {
    pub fn gen_code(&self) -> TokenStream {
        match self {
            Stmt::Block(block) => gen_chained(&block.stmts),
            Stmt::Slot(slot) => gen_slot(slot),
            Stmt::For(for_stmt) => gen_for(for_stmt),
            Stmt::If(if_stmt) => gen_if(if_stmt),
            Stmt::Component(comp) => {
                todo!()
            }
            _ => todo!(),
        }
    }

    pub fn exec_points(&self) -> usize {
        todo!()
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

fn gen_slot(UseSlot { var }: &UseSlot) -> TokenStream {
    quote! {
        #var.merge(#VAR_INPUT_DP)
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
        #PATH_CONTROL_FLOW::execute(|| {
            #PATH_CONTROL_FLOW::repeat(
                ::core::iter::Iterator::map(
                    ::core::iter::IntoIterator::into_iter(#expr),
                    |#pattern| (#get_key, #body)
                )
            )
        })
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
        #PATH_CONTROL_FLOW::execute(|| {
            if #condition {
                #PATH_CONTROL_FLOW::branch(
                    #PATH_CONTROL_FLOW::branch::Branch::A(
                        #then_branch
                    )
                )
            } else {
                #PATH_CONTROL_FLOW::branch(
                    #PATH_CONTROL_FLOW::branch::Branch::B(
                        #else_branch
                    )
                )
            }
        })
    }
}
