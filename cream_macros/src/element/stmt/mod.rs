use std::rc::Rc;

use syn::{Expr, Ident, Type};

use crate::{element::ElementCodegen, expr::StateExpr};

mod parse;
mod to_tokens;

pub struct ElementStmt {
    element: Type,
    props: Vec<(Ident, Expr)>,
    _rename: Option<Ident>,
    style: Expr,
    event_src: Option<Rc<Expr>>,
    event_listeners: Vec<(Type, Expr)>,
    children: Vec<StateExpr<ElementCodegen>>,
}

impl ElementStmt {
    pub fn set_event_src(&mut self, expr: Rc<Expr>) {
        self.event_src = Some(expr);
    }

    pub fn children_mut(&mut self) -> &mut [StateExpr<ElementCodegen>] {
        &mut self.children
    }
}
