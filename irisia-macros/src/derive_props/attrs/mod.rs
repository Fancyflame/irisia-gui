use proc_macro2::Ident;
use syn::{ExprPath, Type, Visibility};

mod parse_field_attr;
mod parse_struct_attr;

pub struct StructAttr {
    pub updater_name: Ident,
    pub visibility: Visibility,
}

pub enum FieldAttr {
    Skipped,
    Normal {
        value_resolver: FieldResolver,
        default_behavior: FieldDefault,
        options: FieldOptions,
    },
}

#[derive(Clone)]
pub enum FieldResolver {
    MoveOwnership,
    CallUpdater,
    ReadStyle,
    WithFn { path: ExprPath, arg_type: Type },
}

#[derive(Clone)]
pub enum FieldDefault {
    MustInit,
    Default,
    DefaultWith(ExprPath),
}

#[derive(Clone)]
pub struct FieldOptions {
    rename: Option<Ident>,
}
