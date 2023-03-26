use std::rc::Rc;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::ParseStream, Error, Expr, Result};

use crate::expr::{
    state_block::{parse_stmts, stmts_to_tokens},
    state_command::{StateCommand, StateCommandBody},
    StateExpr, VisitUnit,
};

use super::{
    cmd::{ElementCommand, InitCommand},
    ElementCodegen,
};

pub fn build(input: ParseStream) -> Result<TokenStream> {
    let mut stmts = parse_stmts::<ElementCodegen>(input)?;
    let (args, rest) = match stmts.split_first_mut() {
        Some((
            StateExpr::Command(StateCommand {
                body: StateCommandBody::Custom(ElementCommand::Init(InitCommand { args })),
                ..
            }),
            rest,
        )) => (args, rest),
        _ => return Err(input.error("missing `init` command")),
    };

    let mut args = args.into_iter();

    if let Some(setter) = args.next() {
        walk_tree(rest, &Rc::new(setter.clone()))?;
    }

    if let Some(expr) = args.next() {
        return Err(Error::new_spanned(expr, "unused argument found here"));
    }

    let mut stmts_expr = TokenStream::new();
    stmts_to_tokens(&mut stmts_expr, rest);
    Ok(quote! {{
        use cream_core::structure::StructureBuilder;
        #stmts_expr
    }})
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
    match expr {
        StateExpr::Raw(r) => {
            r.set_event_src(event_src.clone());
            walk_tree(r.children_mut(), event_src)?;
        }

        StateExpr::Command(StateCommand {
            body: StateCommandBody::Custom(command),
            span,
        }) => match command {
            ElementCommand::Init(_) => {
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
