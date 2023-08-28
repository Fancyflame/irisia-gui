use std::collections::HashSet;

use case::CaseExt;
use proc_macro2::{Span, TokenStream};
use quote::format_ident;
use syn::{
    parse_quote, punctuated::Punctuated, spanned::Spanned, visit::Visit, Error, Fields,
    GenericParam, Generics, Ident, ItemStruct, Result, Type, Visibility,
};

use crate::derive_props::attrs::StructAttr;

use self::impl_::{impl_default, make_struct, regenerate_origin_struct, set_props};

mod attrs;
mod impl_;

struct GenHelper<'a> {
    item: &'a ItemStruct,
    target_struct: Ident,
    vis: Visibility,
    updater_generics: Generics,
}

struct HandledField<'a> {
    ident: &'a Ident,
    ty: &'a Type,
}

impl<'a> GenHelper<'a> {
    fn new(item: &'a ItemStruct, struct_attr: &StructAttr) -> Self {
        Self {
            item,
            target_struct: struct_attr.updater_name.clone(),
            vis: struct_attr.visibility.clone(),
            updater_generics: new_generics(&item),
        }
    }

    fn field_iter(&self) -> impl Iterator<Item = (&Ident, &Type)> {
        self.item
            .fields
            .iter()
            .map(|field| (field.ident.as_ref().unwrap(), &field.ty))
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
    let helper = GenHelper::new(&item, &struct_attr);

    let mut output = regenerate_origin_struct(&helper);
    output.extend(make_struct(&helper));
    output.extend(impl_default(&helper));
    output.extend(set_props(&helper));

    println!("{}", output.to_string());

    Ok(output)
}
