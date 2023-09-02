use std::collections::HashSet;

use case::CaseExt;
use proc_macro2::TokenStream;
use quote::format_ident;
use syn::{
    parse_quote, punctuated::Punctuated, spanned::Spanned, visit::Visit, Error, Fields,
    GenericParam, Generics, Ident, ItemStruct, Result, Type,
};

use crate::derive_props::attrs::StructAttr;

use self::{
    attrs::FieldAttr, impl_miscellaneous::impl_miscellaneous, impl_update_with::impl_update_with,
};

mod attrs;
mod impl_miscellaneous;
mod impl_update_with;

#[non_exhaustive]
struct GenHelper<'a> {
    item: &'a ItemStruct,
    struct_attr: StructAttr,
    updater_generics: Generics,
    fields: Vec<HandledField<'a>>,
}

struct HandledField<'a> {
    ident: &'a Ident,
    ty: &'a Type,
    attr: FieldAttr,
}

impl<'a> GenHelper<'a> {
    fn new(item: &'a ItemStruct, struct_attr: StructAttr, fields: Vec<HandledField<'a>>) -> Self {
        Self {
            item,
            updater_generics: new_generics(&item),
            fields,
            struct_attr,
        }
    }

    fn generics_iter(&self) -> impl Iterator<Item = &Ident> + Clone {
        self.updater_generics
            .params
            .iter()
            .map(|param| match param {
                GenericParam::Type(t) => &t.ident,
                _ => unreachable!(
                    "any `GenericParam` other than `GenericParam::Type` is not allowed"
                ),
            })
    }
}

fn new_generics<'a>(stru: &ItemStruct) -> Generics {
    let field_types: HashSet<&Ident> = {
        struct IdentVisitor<'ast>(HashSet<&'ast Ident>);
        impl<'ast> Visit<'ast> for IdentVisitor<'ast> {
            fn visit_ident(&mut self, i: &'ast Ident) {
                self.0.insert(i);
            }
        }

        let mut ident_visitor = IdentVisitor(HashSet::new());
        syn::visit::visit_item_struct(&mut ident_visitor, stru);
        ident_visitor.0
    };

    let param_iter = stru.fields.iter().map(|field| {
        let raw_id = field
            .ident
            .as_ref()
            .expect("expected named field")
            .to_string()
            .to_camel();

        let mut id = format_ident!("Prop{raw_id}");
        loop {
            if !field_types.contains(&id) {
                let gp: GenericParam = parse_quote!(#id);
                break gp;
            }
            id = format_ident!("{id}Generic");
        }
    });

    Generics {
        params: Punctuated::from_iter(param_iter),
        ..Default::default()
    }
}

pub fn props(attr: TokenStream, item: ItemStruct) -> Result<TokenStream> {
    if !matches!(item.fields, Fields::Named(_)) {
        return Err(Error::new(
            item.span(),
            "expected a struct with named fields",
        ));
    }

    let struct_attr = StructAttr::parse_from(attr, item.vis.clone())?;

    let field_attrs: Vec<HandledField> = {
        let mut attrs = Vec::new();
        for field in item.fields.iter() {
            attrs.push(HandledField {
                ident: &field.ident.as_ref().unwrap(),
                ty: &field.ty,
                attr: FieldAttr::parse_from(&field.attrs, field.span())?,
            });
        }
        attrs
    };

    let helper = GenHelper::new(&item, struct_attr, field_attrs);

    let mut output = impl_miscellaneous(&helper);
    output.extend(impl_update_with(&helper));
    Ok(output)
}
