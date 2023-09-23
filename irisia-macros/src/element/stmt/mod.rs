use syn::{Expr, Ident, Type};

use crate::{element::ElementCodegen, expr::StateExpr};

mod parse;
mod to_tokens;

pub struct ElementStmt {
    element: Type,
    props: Vec<(Ident, Expr)>,
    style: Option<Expr>,
    oncreate: Option<Expr>,
    children: Vec<StateExpr<ElementCodegen>>,
}
