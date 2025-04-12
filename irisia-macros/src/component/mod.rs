use syn::{Expr, Ident, Type};

mod parse;
mod to_tokens;

pub struct BuildMacro(Vec<Stmt>);

struct FieldDefinition {
    name: Ident,
    field_type: FieldType,
}

enum FieldType {
    Value { from_ty: Type, to_ty: Type },
    Model,
}

enum Stmt {
    If(IfStmt),
    Match(MatchStmt),
    For(ForStmt),
    While(WhileStmt),
    Component(Component),
    Block(BlockStmt),
    UseExpr(Expr),
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

struct Component {
    type_path: syn::Path,
    fields: Vec<FieldAssignment>,
    body: Vec<Stmt>,
}

struct FieldAssignment {
    name: Ident,
    value: Expr,
    method: FieldAssignMethod,
}

enum FieldAssignMethod {
    HostingSignal,
    Direct,
}
