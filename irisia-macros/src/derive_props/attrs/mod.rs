use proc_macro2::Ident;
use syn::{Expr, ExprPath, Type, Visibility};

mod parse_field_attr;
mod parse_struct_attr;

pub struct StructAttr {
    pub updater_name: Ident,
    pub visibility: Visibility,
}

pub struct FieldAttr {
    pub value_resolver: FieldResolver,
    pub default_behavior: FieldDefault,
    pub options: FieldOptions,
}

impl FieldAttr {
    pub fn is_std_style_input(&self) -> bool {
        matches!(
            self.value_resolver,
            FieldResolver::ReadStyle { as_std_input: true }
        )
    }
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

#[derive(Clone)]
pub struct FieldOptions {
    pub rename: Option<Ident>,
}
