use attr_parser_fn::{
    meta::{conflicts, key_value, path_only, ParseMetaExt},
    ParseArgs, ParseAttrTrait,
};

use syn::{Attribute, Error, Expr, Field, Ident, Result};

pub enum Defaulter {
    Required,
    Default,
    DefaultWith(Expr),
}

pub struct FieldProps {
    pub field: Field,
    pub rename: Option<Ident>,
    pub defaulter: Defaulter,
}

fn is_props_attr(attr: &&Attribute) -> bool {
    attr.meta.path().is_ident("props")
}

impl FieldProps {
    pub fn parse(field: Field) -> Result<Self> {
        let mut attrs = field.attrs.iter();

        let Some(attr) = attrs.find(is_props_attr) else {
            return Ok(FieldProps {
                field,
                rename: None,
                defaulter: Defaulter::Required,
            });
        };

        if let Some(duplicated_attr) = attrs.find(is_props_attr) {
            return Err(Error::new_spanned(
                duplicated_attr,
                "cannot declare attribute `props` duplicatedly",
            ));
        }

        let (rename, defaulter) = ParseArgs::new()
            .meta((
                ("rename", key_value::<Ident>()).optional(),
                conflicts((
                    ("default", path_only()).value(Defaulter::Default),
                    ("default", key_value::<Expr>()).map(Defaulter::DefaultWith),
                ))
                .optional()
                .map(|default| default.unwrap_or(Defaulter::Required)),
            ))
            .parse_attrs(attr)?
            .meta;

        Ok(FieldProps {
            field,
            rename,
            defaulter,
        })
    }
}
