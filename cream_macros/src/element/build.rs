use std::rc::Rc;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::ParseStream, Error, Expr, Result};

use crate::expr::{
    state_block::{parse_stmts, stmts_to_tokens},
    state_command::{StateCommand, StateCommandBody},
    StateExpr, VisitUnit,
};

use super::{cmd::ElementCommand, ElementCodegen};

pub fn build(input: ParseStream, render: bool) -> Result<TokenStream> {
    let mut stmts = parse_stmts::<ElementCodegen>(input)?;
    let (init, span, stmts_expr) = match stmts.split_first_mut() {
        Some((
            StateExpr::Command(StateCommand {
                span,
                body:
                    StateCommandBody::Custom(
                        init @ ElementCommand::InitRender { .. }
                        | init @ ElementCommand::InitBuild { .. },
                    ),
            }),
            rest,
        )) => {
            let event_src = match init {
                ElementCommand::InitBuild { event_src }
                | ElementCommand::InitRender { event_src, .. } => Rc::new(event_src.clone()),
                _ => unreachable!(),
            };
            walk_tree(rest, &event_src)?;
            let mut stmts_expr = TokenStream::new();
            stmts_to_tokens(&mut stmts_expr, rest);
            (init, span, stmts_expr)
        }
        _ => return Err(input.error("missing `init` command")),
    };

    match (render, init) {
        (
            true,
            ElementCommand::InitRender {
                event_src: _,
                cache_box,
                render_content,
            },
        ) => Ok(quote! {
            ::cream_core::structure::Node::fin(#stmts_expr, #cache_box, #render_content)
        }),

        (false, ElementCommand::InitBuild { .. }) => Ok(stmts_expr),

        (true, ElementCommand::InitBuild { .. }) => Err(Error::new(
            span.clone(),
            "complete `init` command is required: `@init(_, _, _)`",
        )),

        (false, ElementCommand::InitRender { .. }) => Err(Error::new(
            span.clone(),
            "complete `init` command is unnecessary, consider short form: `@init(_)`",
        )),

        _ => unreachable!(),
    }
}

fn walk_tree(root_exprs: &mut [StateExpr<ElementCodegen>], event_src: &Rc<Expr>) -> Result<()> {
    for expr in root_exprs {
        let mut result = Ok(());
        expr.visit_unit_mut(&mut |expr2| {
            if result.is_ok() {
                result = proc_one(expr2, event_src);
            }
        });

        result?;
    }
    Ok(())
}

fn proc_one(expr: &mut StateExpr<ElementCodegen>, event_src: &Rc<Expr>) -> Result<()> {
    println!("set raw");
    match expr {
        StateExpr::Raw(r) => {
            r.set_event_src(event_src.clone());
            walk_tree(r.children_mut(), event_src)?;
        }

        StateExpr::Command(StateCommand {
            body: StateCommandBody::Custom(command),
            span,
        }) => match command {
            ElementCommand::InitRender { .. } | ElementCommand::InitBuild { .. } => {
                return Err(Error::new(
                    span.clone(),
                    "`init` command can only be used at start of the root",
                ));
            }

            ElementCommand::Slot(_) => {}
        },

        _ => {}
    }
    Ok(())
}
