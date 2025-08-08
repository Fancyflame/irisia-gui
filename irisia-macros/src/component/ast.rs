use proc_macro2::TokenStream;
use syn::{Expr, Ident};

pub enum Stmt {
    If(IfStmt),
    Match(MatchStmt),
    For(ForStmt),
    While(WhileStmt),
    Component(ComponentStmt),
    Block(BlockStmt),
    UseExpr(UseExprStmt),
}

pub struct IfStmt {
    pub condition: syn::Expr,
    pub then_branch: BlockStmt,
    pub else_branch: Option<Box<Stmt>>,
}

pub struct MatchStmt {
    pub expr: syn::Expr,
    pub arms: Vec<MatchArm>,
}

pub struct MatchArm {
    pub pattern: syn::Pat,
    pub guard: Option<Expr>,
    pub body: Stmt,
}

pub struct ForStmt {
    pub pattern: syn::Pat,
    pub expr: syn::Expr,
    pub get_key: Option<syn::Expr>,
    pub body: BlockStmt,
}

pub struct WhileStmt {
    pub condition: syn::Expr,
    pub body: BlockStmt,
}

pub struct BlockStmt {
    pub stmts: Vec<Stmt>,
}

pub struct ComponentStmt {
    pub comp_type: syn::Path,
    pub fields: Vec<FieldAssignment<Ident>>,
    pub child_data: Option<Expr>,
    pub body: Vec<Stmt>,
}

pub struct UseExprStmt {
    pub value: Option<TokenStream>,
}

pub struct FieldAssignment<Name> {
    pub name: Name,
    pub value: Expr,
    pub decoration: FieldDecoration,
}

#[derive(Clone, Copy)]
pub enum FieldDecoration {
    None,
    DirectAssign,
    Event,
}
