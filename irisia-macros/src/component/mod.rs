use proc_macro2::TokenStream;
use syn::{Expr, Ident, Path};

mod parse;
mod to_tokens;

pub struct BuildMacro {
    stmts: Vec<Stmt>,
    virtual_parent: Option<Path>,
}

enum Stmt {
    If(IfStmt),
    Match(MatchStmt),
    For(ForStmt),
    While(WhileStmt),
    Component(ComponentStmt),
    Block(BlockStmt),
    UseExpr(UseExprStmt),
}

struct IfStmt {
    condition: syn::Expr,
    then_branch: BlockStmt,
    else_branch: Option<Box<Stmt>>,
}

struct MatchStmt {
    expr: syn::Expr,
    arms: Vec<MatchArm>,
}

struct MatchArm {
    pattern: syn::Pat,
    guard: Option<Expr>,
    body: Stmt,
}

struct ForStmt {
    pattern: syn::Pat,
    expr: syn::Expr,
    get_key: Option<syn::Expr>,
    body: BlockStmt,
}

struct WhileStmt {
    condition: syn::Expr,
    body: BlockStmt,
}

struct BlockStmt {
    stmts: Vec<Stmt>,
}

struct ComponentStmt {
    comp_type: syn::Path,
    fields: Vec<FieldAssignment>,
    body: Vec<Stmt>,
}

struct UseExprStmt {
    value: Option<TokenStream>,
}

struct FieldAssignment {
    name: Ident,
    value: Expr,
    method: FieldAssignMethod,
}

enum FieldAssignMethod {
    ParentProp,
    HostingSignal,
    Direct,
}

fn check_has_parent_props_assigned(stmt: &Stmt) -> bool {
    fn check_sliced(stmts: &[Stmt]) -> bool {
        stmts.iter().any(check_has_parent_props_assigned)
    }

    fn check_fields(fa: &[FieldAssignment]) -> bool {
        fa.iter()
            .any(|fa| matches!(fa.method, FieldAssignMethod::ParentProp))
    }

    match stmt {
        Stmt::Block(block) => check_sliced(&block.stmts),
        Stmt::Component(comp) => {
            check_fields(&comp.fields)
            // do not check the body: `check_sliced(&comp.body)`
        }
        Stmt::For(for_stmt) => check_sliced(&for_stmt.body.stmts),
        Stmt::If(if_stmt) => {
            check_sliced(&if_stmt.then_branch.stmts)
                || match &if_stmt.else_branch {
                    Some(stmt) => check_has_parent_props_assigned(&stmt),
                    None => false,
                }
        }
        Stmt::Match(match_stmt) => match_stmt
            .arms
            .iter()
            .any(|arm| check_has_parent_props_assigned(&arm.body)),
        Stmt::UseExpr(_) => false,
        Stmt::While(while_stmt) => check_sliced(&while_stmt.body.stmts),
    }
}
