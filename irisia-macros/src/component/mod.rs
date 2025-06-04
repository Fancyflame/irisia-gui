use proc_macro2::TokenStream;
use syn::{Expr, Ident};

mod parse;
mod to_tokens;

pub struct BuildMacro {
    stmts: Vec<Stmt>,
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
    fields: Vec<FieldAssignment<Ident>>,
    child_data: Option<Expr>,
    body: Vec<Stmt>,
}

struct UseExprStmt {
    value: Option<TokenStream>,
}

struct FieldAssignment<Name> {
    name: Name,
    value: Expr,
    method: FieldAssignMethod,
}

enum FieldAssignMethod {
    HostingSignal,
    Direct,
}
