use std::collections::HashMap;

use proc_macro2::Ident;
use syn::{Expr, Type, Visibility};

use self::parse::ElementStmt;

pub mod parse;

struct DataDec {
    ty: Type,
    init: Expr,
}

struct PropDec {
    ty: Type,
    default: Option<Expr>, // if not, prop is required
}

pub struct App {
    vis: Visibility,
    ident: Ident,
    data: HashMap<Ident, DataDec>,
    props: HashMap<Ident, PropDec>,
    watch: HashMap<Ident, Expr>,
    child_node: Option<ElementStmt>,
    on_mount: Option<Expr>,
}
