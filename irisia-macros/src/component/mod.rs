use syn::{Expr, Ident, Type};

mod codegen;
mod parse;

pub struct DomMacro {
    name: Ident,
    generics: syn::Generics,
    fields: Vec<FieldDefinition>,
    body: Vec<Stmt>,
}

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
    Slot(UseSlot),
    Component(Component),
    Block(BlockStmt),
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
    body: BlockStmt,
}

struct ForStmt {
    pattern: syn::Pat,
    expr: syn::Expr,
    body: BlockStmt,
}

struct WhileStmt {
    condition: syn::Expr,
    body: BlockStmt,
}

struct UseSlot {
    var: Ident,
}

struct BlockStmt {
    stmts: Vec<Stmt>,
}

struct Component {
    path: syn::Path,
    fields: Vec<FieldAssignment>,
    body: Vec<Stmt>,
}

enum FieldAssignment {
    Value { name: Ident, value: syn::Expr },
    Model { name: Ident, tree: Stmt },
}
