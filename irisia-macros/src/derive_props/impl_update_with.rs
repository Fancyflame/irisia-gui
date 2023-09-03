use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{ItemStruct, Type};

use crate::derive_props::attrs::{FieldAttr, StructAttr};

use super::{
    attrs::{FieldDefault, FieldResolver},
    GenHelper, HandledField,
};

pub(super) fn impl_update_with(helper: &GenHelper) -> TokenStream {
    let update_with = generate_update_with(helper);
    let create_with = generate_create_with(helper);
    let where_clause = generate_where_clause(helper);
    let update_result_item = make_update_reuslt(helper);

    let GenHelper {
        item: ItemStruct {
            ident: origin_struct,
            ..
        },
        struct_attr:
            StructAttr {
                updater_name,
                update_result,
                ..
            },
        updater_generics,
        ..
    } = helper;

    quote! {
        impl #updater_generics irisia::element::props::PropsUpdateWith<#updater_name #updater_generics>
            for #origin_struct
        where
            #where_clause
        {
            type UpdateResult = #update_result;

            #update_with
            #create_with
        }

        #update_result_item
    }
}

fn get_resolver(fr: &FieldResolver, field_type: &Type, use_expr: bool) -> TokenStream {
    match fr {
        FieldResolver::CallUpdater => quote!(irisia::element::props::CallUpdater),
        FieldResolver::Custom(custom) => quote!(#custom),
        FieldResolver::MoveOwnership => quote!(irisia::element::props::MoveOwnership),
        FieldResolver::ReadStyle { as_std_input: _ } => quote!(irisia::element::props::ReadStyle),
        FieldResolver::WithFn { arg_type, path } => {
            let ty = quote!(fn(#arg_type) -> #field_type);
            if use_expr {
                quote!((#path as #ty))
            } else {
                quote!(#ty)
            }
        }
    }
}

fn generate_where_clause(helper: &GenHelper) -> TokenStream {
    let mut output = quote!();
    for (field, generic_type) in helper.fields.iter().zip(helper.generics_iter()) {
        let field_type = field.ty;
        let resolver = get_resolver(&field.attr.value_resolver, field_type, false);

        let must_init = if let FieldDefault::MustInit = field.attr.default_behavior {
            quote!(Def = irisia::element::props::PropInitialized<#field_type>,)
        } else {
            quote!()
        };

        quote! {
            #resolver: irisia::element::props::HelpUpdate<#field_type, #generic_type, #must_init>,
        }
        .to_tokens(&mut output);
    }
    output
}

fn generate_update_with(helper: &GenHelper) -> TokenStream {
    fn update_field(
        HandledField {
            ident,
            ty,
            attr: FieldAttr { value_resolver, .. },
        }: &HandledField,
        equality_matters: TokenStream,
    ) -> TokenStream {
        let resolver = get_resolver(value_resolver, &ty, true);
        quote! {
            irisia::element::props::HelpUpdate::update(
                &#resolver,
                &mut self.#ident,
                __irisia_updater.#ident,
                #equality_matters
            )
        }
    }

    let GenHelper {
        updater_generics,
        fields,
        struct_attr:
            StructAttr {
                updater_name,
                update_result,
                default_watch,
                ..
            },
        ..
    } = helper;

    let unwatched = fields.iter().filter_map(|f| {
        if f.attr.watch.is_some() {
            return None;
        }

        let tokens = match default_watch {
            Some(dw) if dw.exclude.contains(f.ident) => {
                let uf = update_field(f, quote!(false));
                quote!(#uf;)
            }
            _ => {
                let uf = update_field(f, quote!(__irisia_equality_matters));
                quote!(__irisia_equality_matters &= #uf;)
            }
        };

        Some(tokens)
    });

    let watched = fields.iter().filter_map(|f| {
        let Some(field_changed) = &f.attr.watch
        else {
            return None;
        };

        let value = update_field(f, quote!(true));

        let out = match default_watch {
            Some(dw) if dw.exclude.contains(f.ident) => quote! {
                #field_changed: {
                    let __irisia_changed = #value;
                    __irisia_equality_matters &= __irisia_changed;
                    !__irisia_changed
                },
            },
            _ => quote!(#field_changed: !#value,),
        };

        Some(out)
    });

    let add_global_change = default_watch.as_ref().map(|dw| {
        let group_name = &dw.group_name;
        quote!(#group_name: !__irisia_equality_matters,)
    });

    quote! {
        fn props_update_with(
            &mut self,
            __irisia_updater: #updater_name #updater_generics,
            mut __irisia_equality_matters: bool,
        ) -> #update_result {
            #(#unwatched)*

            #update_result {
                #add_global_change
                #(#watched)*
            }
        }
    }
}

fn generate_create_with(helper: &GenHelper) -> TokenStream {
    let GenHelper {
        updater_generics,
        fields,
        struct_attr: StructAttr { updater_name, .. },
        ..
    } = helper;

    let iter = fields.iter().map(|HandledField { ident, ty, attr }| {
        let resolver = get_resolver(&attr.value_resolver, &ty, true);

        let maybe_created = quote! {
            irisia::element::props::HelpCreate::create(
                &#resolver,
                __irisia_updater.#ident
            )
        };

        fn use_defaulter(
            maybe_created: TokenStream,
            default_value: impl ToTokens,
            ret_type: &Type,
        ) -> TokenStream {
            quote! {
                irisia::element::props::Defaulter::with_defaulter(
                    #maybe_created,
                    (|| -> #ret_type { #default_value } as fn() -> #ret_type)
                )
            }
        }

        let final_expr = match &attr.default_behavior {
            FieldDefault::Default => use_defaulter(
                maybe_created,
                quote! {
                    ::std::default::Default::default()
                },
                ty,
            ),
            FieldDefault::DefaultWith(def) => use_defaulter(maybe_created, def, ty),
            FieldDefault::MustInit => quote!(#maybe_created.must_be_initialized()),
        };

        quote! {
            #ident: #final_expr,
        }
    });

    quote! {
        fn props_create_with(
            __irisia_updater: #updater_name #updater_generics
        ) -> Self {
            Self {
                #(#iter)*
            }
        }
    }
}

fn make_update_reuslt(helper: &GenHelper) -> TokenStream {
    let GenHelper {
        struct_attr:
            StructAttr {
                visibility,
                update_result,
                default_watch,
                ..
            },
        fields,
        ..
    } = helper;

    let mut default_watch = default_watch.as_ref();

    let field_iter = std::iter::from_fn(|| default_watch.take().map(|w| &w.group_name))
        .chain(fields.iter().filter_map(|f| f.attr.watch.as_ref()));

    quote! {
        #visibility struct #update_result {
            #(pub #field_iter: bool,)*
        }
    }
}
