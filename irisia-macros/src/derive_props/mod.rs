use std::collections::HashSet;

use case::CaseExt;
use proc_macro2::TokenStream;
use quote::format_ident;
use syn::{
    parse_quote, punctuated::Punctuated, spanned::Spanned, visit::Visit, Error, Fields,
    GenericParam, Generics, Ident, ItemStruct, Result, Type, Visibility,
};

use crate::derive_props::attrs::StructAttr;

use self::{
    attrs::FieldAttr,
    impl_miscellaneous::{impl_default, make_struct, regenerate_origin_struct, set_props},
    impl_update_with::impl_update_with,
};

mod attrs;
mod impl_miscellaneous;
mod impl_update_with;

struct GenHelper<'a> {
    item: &'a ItemStruct,
    target_struct: &'a Ident,
    vis: Visibility,
    updater_generics: Generics,
    fields: Vec<HandledField<'a>>,
}

struct HandledField<'a> {
    ident: &'a Ident,
    ty: &'a Type,
    attr: FieldAttr,
}

impl<'a> GenHelper<'a> {
    fn new(
        item: &'a ItemStruct,
        struct_attr: &'a StructAttr,
        fields: Vec<HandledField<'a>>,
    ) -> Self {
        Self {
            item,
            target_struct: &struct_attr.updater_name,
            vis: struct_attr.visibility.clone(),
            updater_generics: new_generics(&item),
            fields,
        }
    }

    fn generics_iter(&self) -> impl Iterator<Item = &Ident> {
        self.updater_generics.type_params().map(|p| &p.ident)
    }

    fn no_fields(&self) -> bool {
        self.updater_generics.params.is_empty()
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

    let helper = GenHelper::new(&item, &struct_attr, field_attrs);

    let mut output = regenerate_origin_struct(&helper);
    output.extend(make_struct(&helper));
    output.extend(impl_default(&helper));
    output.extend(set_props(&helper));
    output.extend(impl_update_with(&helper));

    Ok(output)
}
