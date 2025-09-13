use proc_macro2::TokenStream;
use quote::format_ident;

use crate::build_macro::{
    ast::FieldValue,
    to_tokens::use_component::low_level::{LlExpr, LlField, LlGenerator},
};

use super::{ComponentStmt, FieldAssignment, GenerationEnv};

mod low_level;

impl GenerationEnv {
    pub(super) fn gen_component(
        &self,
        ComponentStmt {
            comp_type,
            fields: all_fields,
            body,
            child_data,
        }: &ComponentStmt,
    ) -> TokenStream {
        let mut ll_fields: Vec<LlField> = Vec::with_capacity(all_fields.len() + 1);

        for FieldAssignment { name, value } in all_fields {
            match value {
                FieldValue::DirectAssign(d) => ll_fields.push(LlField {
                    name,
                    expr: LlExpr::UserDefined(d),
                    mode: low_level::Mode::Direct,
                }),
                FieldValue::Proxied(p) => ll_fields.push(LlField {
                    name,
                    expr: LlExpr::UserDefined(p),
                    mode: low_level::Mode::Proxied,
                }),
                _ => unimplemented!(),
            }
        }

        let mut _children_ident = None;
        if !body.is_empty() {
            ll_fields.push(LlField {
                name: _children_ident.insert(format_ident!("children")),
                expr: LlExpr::Complex(GenerationEnv {}.gen_rc_chained(&body)),
                mode: low_level::Mode::Proxied,
            });
        };

        let ll_generator = LlGenerator {
            component_path: comp_type,
            fields: ll_fields,
            child_data: child_data.as_ref(),
        };

        ll_generator.generate()
    }
}
