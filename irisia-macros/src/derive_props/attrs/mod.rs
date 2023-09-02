use std::collections::HashSet;

use proc_macro2::Ident;
use syn::{Expr, ExprPath, Type, Visibility};

mod parse_field_attr;
mod parse_struct_attr;

pub struct StructAttr {
    pub updater_name: Ident,
    pub visibility: Visibility,
    pub update_result: Ident,
    pub default_watch: Option<DefaultWatch>,
}

pub struct DefaultWatch {
    pub group_name: Ident,
    pub exclude: HashSet<Ident>,
}

pub struct FieldAttr {
    pub value_resolver: FieldResolver,
    pub default_behavior: FieldDefault,
    pub rename: Option<Ident>,
    pub watch: Option<Ident>,
}

#[derive(Clone)]
pub enum FieldResolver {
    MoveOwnership,
    CallUpdater,
    ReadStyle { as_std_input: bool },
    WithFn { path: ExprPath, arg_type: Type },
    Custom(Type),
}

#[derive(Clone)]
pub enum FieldDefault {
    MustInit,
    Default,
    DefaultWith(Expr),
}

impl FieldAttr {
    pub fn is_std_style_input(&self) -> bool {
        matches!(
            self.value_resolver,
            FieldResolver::ReadStyle { as_std_input: true }
        )
    }
}
