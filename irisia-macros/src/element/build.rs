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
    let stmts = match stmts.split_first_mut() {
        Some((
            StateExpr::Command(StateCommand {
                body: StateCommandBody::Custom(ElementCommand::Init(InitCommand { args })),
                ..
            }),
            rest,
        )) => {
            let mut args = args.into_iter();
            let mut slot = args.next().map(|ex| ex.clone());
            let event_dispatcher = args.next().map(|ex| Rc::new(ex.clone()));
            if let Some(expr) = args.next() {
                return Err(Error::new_spanned(expr, "unused argument found here"));
            }
            walk_tree(rest, &event_dispatcher, &mut slot)?;
            rest
        }
        _ => &mut stmts,
    };

    let mut stmts_tokened = TokenStream::new();
    stmts_to_tokens(&mut stmts_tokened, stmts);
    Ok(quote! {{
        use irisia::structure::StructureBuilder;
        #stmts_tokened
    }})
}

fn walk_tree(
    root_exprs: &mut [StateExpr<ElementCodegen>],
    event_dispatcher: &Option<Rc<Expr>>,
    slot: &mut Option<Expr>,
) -> Result<()> {
    for expr in root_exprs {
        let mut result = Ok(());
        expr.visit_unit_mut(&mut |expr2| {
            if result.is_ok() {
                result = handle_one(expr2, event_dispatcher, slot);
            }
        });

        result?;
    }
    Ok(())
}

fn handle_one(
    expr: &mut StateExpr<ElementCodegen>,
    event_dispatcher: &Option<Rc<Expr>>,
    slot: &mut Option<Expr>,
) -> Result<()> {
    match expr {
        StateExpr::Raw(r) => {
            if let Some(ed) = event_dispatcher {
                r.set_event_dispatcher(ed.clone());
            }
            walk_tree(r.children_mut(), event_dispatcher, slot)?;
        }

        StateExpr::Command(StateCommand {
            body: StateCommandBody::Custom(command),
            span,
        }) => match command {
            ElementCommand::Init(_) => {
                return Err(Error::new(
                    *span,
                    "`init` command can only be used at start of the root",
                ));
            }

            ElementCommand::Slot(slot_cmd) => match slot.take() {
                Some(expr) => *slot_cmd = Some(expr),
                None => {
                    return Err(Error::new(
                        *span,
                        "no slot provided in `@init` command or has been used",
                    ))
                }
            },
        },

        _ => {}
    }
    Ok(())
}
