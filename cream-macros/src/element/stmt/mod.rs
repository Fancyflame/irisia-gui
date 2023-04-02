use std::rc::Rc;

use syn::{Expr, Ident, Type};

use crate::{element::ElementCodegen, expr::StateExpr};

mod parse;
mod to_tokens;

pub struct ElementStmt {
    element: Type,
    props: Vec<(Ident, Expr)>,
    rename: Option<Ident>,
    style: Expr,
    event_dispatcher: Option<Rc<Expr>>,
    event_emitting_key: Option<Expr>,
    children: Vec<StateExpr<ElementCodegen>>,
}

impl ElementStmt {
    pub fn set_event_dispatcher(&mut self, expr: Rc<Expr>) {
        self.event_dispatcher = Some(expr);
    }

    pub fn children_mut(&mut self) -> &mut [StateExpr<ElementCodegen>] {
        &mut self.children
    }
}
