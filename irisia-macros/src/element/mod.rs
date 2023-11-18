use crate::expr::StmtTree;

pub mod parse;

pub struct ElementCodegen;

impl StmtTree for ElementCodegen {
    type Command = ();
    type Stmt = parse::ElementStmt;

    const MUST_IN_BLOCK: bool = false;
}
