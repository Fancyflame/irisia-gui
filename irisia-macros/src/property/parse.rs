use super::{FieldInput, MacroInput};
use case::CaseExt;
use darling::{FromDeriveInput, FromField, ast, util::Flag};
use quote::format_ident;
use syn::{DeriveInput, Error, Ident, LitStr, Result, Visibility};

type MacroFields = ast::Data<(), FieldOpts>;

pub(super) fn parse_derive(input: DeriveInput) -> Result<MacroInput> {
    let opts = match MacroOpts::from_derive_input(&input) {
        Ok(v) => v,
        Err(e) => return Err(Error::new(e.span(), e)),
    };

    let field_opts = match opts.data {
        ast::Data::Struct(fields) => fields.fields,
        _ => unreachable!(),
    };

    let mut extend_field: Option<(usize, Ident)> = None;

    let mut fields = Vec::with_capacity(field_opts.len());
    for (index, field) in field_opts.into_iter().enumerate() {
        let rename: Option<Ident> = if let Some(rename) = &field.rename {
            Some(rename.parse()?)
        } else {
            None
        };

        let prop_type = field.get_generic_name();
        let ident = field.ident.unwrap();

        if field.extend.is_present() {
            match extend_field {
                Some((_, used_by)) => {
                    return Err(Error::new_spanned(
                        &ident,
                        format!(
                            "cannot extend this field, because this property has already extended by `{used_by}`"
                        ),
                    ));
                }
                None => extend_field = Some((index, ident.clone())),
            }
        }

        fields.push(FieldInput {
            ident,
            ty: field.ty,
            rename,
            prop_type,
        });
    }

    let template_ident = format_ident!("__IrisiaProp{}", opts.ident, span = opts.ident.span());

    Ok(MacroInput {
        vis: opts.vis,
        struct_ident: opts.ident,
        generics: opts.generics,
        template_ident,
        extend_field_index: extend_field.map(|(index, _)| index),
        fields,
    })
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(prop), supports(struct_named))]
struct MacroOpts {
    vis: Visibility,
    ident: syn::Ident,
    generics: syn::Generics,
    data: MacroFields,
}

#[derive(Debug, FromField)]
#[darling(attributes(prop))]
struct FieldOpts {
    ident: Option<syn::Ident>,
    ty: syn::Type,

    extend: Flag,

    #[darling(default)]
    rename: Option<LitStr>,
}

impl FieldOpts {
    fn get_generic_name(&self) -> Ident {
        let ident = self.ident.as_ref().unwrap();
        let camel_case = ident.to_string().to_camel();
        format_ident!("__IrisiaType{camel_case}", span = ident.span())
    }
}
