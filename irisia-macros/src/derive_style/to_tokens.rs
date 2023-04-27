use std::collections::HashMap;

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse_quote, punctuated::Punctuated, Error, Expr, Fields, Generics, Ident, Index, Member,
    Token, TypeTuple, Visibility,
};

use super::{
    anaylyze_fields::{ExtractResult, FieldMetadata},
    parse_paths::Segment,
};

pub fn write_stream(
    tokens: &mut TokenStream,
    fields: &Fields,
    variant: Option<&Ident>,
    vis: &Visibility,
    ident: &Ident,
    generics: &Generics,
    ExtractResult {
        paths,
        metadatas,
        impl_default,
    }: ExtractResult,
) {
    let (impl_gen, type_gen, where_clause) = generics.split_for_impl();

    for (field_member, field_metadata) in metadatas.iter() {
        let func = match option_func(variant, field_member, field_metadata) {
            Some(f) => f,
            None => continue,
        };

        quote! {
            #[automatically_derived]
            impl #impl_gen #ident #type_gen #where_clause {
                #vis #func
            }
        }
        .to_tokens(tokens);
    }

    for path in paths.iter() {
        let (type_tuple, func) = path_func(fields, variant, path, &metadatas);
        quote! {
            #[automatically_derived]
            impl #impl_gen ::std::convert::From<#type_tuple> for #ident #type_gen
            #where_clause
            {
                #func
            }
        }
        .to_tokens(tokens);
    }

    if impl_default {
        let group = do_impl_default(fields, &metadatas);

        let colon2 = variant.map(|_| <Token![::]>::default());
        quote! {
            #[automatically_derived]
            impl #impl_gen ::std::default::Default for #ident #type_gen
            #where_clause
            {
                fn default() -> Self {
                    Self #colon2 #variant #group
                }
            }
        }
        .to_tokens(tokens);
    }
}

fn option_func(
    variant: Option<&Ident>,
    field_member: &Member,
    FieldMetadata {
        ty,
        option,
        option_set_true,
        ..
    }: &FieldMetadata,
) -> Option<TokenStream> {
    let option_name = match option {
        Some(o) => o,
        None => return None,
    };

    let setter = match variant {
        None => quote! {self.#field_member = value},
        Some(variant) => {
            let pattern = match field_member {
                Member::Named(ident) => quote!({ #ident: field, .. }),
                Member::Unnamed(index) => {
                    let iter = std::iter::repeat(<Token![_]>::default()).take(index.index as usize);
                    quote!((#(#iter,)* field, ..))
                }
            };

            quote! {
                match self {
                    Self::#variant #pattern => *field = value,
                    _ => panic!(
                        "option `{}` can only be called on variant `{}`",
                        stringify!(#option_name),
                        stringify!(#variant),
                    )
                }
            }
        }
    };

    let func = if *option_set_true {
        quote! {
            fn #option_name(&mut self) {
                let value = true;
                #setter
            }
        }
    } else {
        quote! {
            fn #option_name(&mut self, value: #ty) {
                #setter
            }
        }
    };

    Some(func)
}

fn path_func(
    fields: &Fields,
    variant: Option<&Ident>,
    path: &[Segment],
    metadatas: &HashMap<Member, FieldMetadata>,
) -> (TypeTuple, TokenStream) {
    let use_default: Expr = parse_quote!(::std::default::Default::default());
    let mut tuple = TypeTuple {
        paren_token: Default::default(),
        elems: Punctuated::new(),
    };
    let mut fields_init: Vec<(&Member, Expr)> = Vec::new();

    let mut index_counter = 0u32;
    let mut index_iter = std::iter::from_fn(|| {
        let r = Some(Index {
            index: index_counter,
            span: Span::call_site(),
        });
        index_counter += 1;
        r
    });

    for seg in path.iter() {
        match seg {
            Segment::Member(member) => {
                tuple.elems.push(
                    metadatas
                        .get(member)
                        .expect("inner error: member not contained in hashmap")
                        .ty
                        .clone(),
                );

                let index = index_iter.next().unwrap();

                fields_init.push((member, parse_quote!(value.#index)));
            }
            Segment::Fn {
                bind,
                path,
                arg_types,
            } => {
                tuple
                    .elems
                    .extend(arg_types.pairs().map(|pair| pair.cloned()));

                let iter = (&mut index_iter).take(arg_types.len());

                fields_init.push((bind, parse_quote!( (#path)(#(value.#iter),*) )))
            }
        }
    }

    'field_loop: for (member, metadata) in metadatas.iter() {
        for seg in path {
            let (Segment::Fn { bind, .. } | Segment::Member(bind)) = seg;
            if bind == member {
                continue 'field_loop;
            }
        }

        let default_behavior = match &metadata.default {
            None => {
                let string = format!(
                    "default behavior of field `{}` is required",
                    member.to_token_stream()
                );
                parse_quote!(::std::compile_error!(#string))
            }
            Some(None) => use_default.clone(),
            Some(Some(expr)) => expr.clone(),
        };

        fields_init.push((member, default_behavior));
    }

    let members = fields_init.iter().map(|x| x.0);
    let exprs = fields_init.iter().map(|x| &x.1);

    let setter = match fields {
        Fields::Named(_) => Some(quote! {
            {#(#members: #exprs,)*}
        }),
        Fields::Unnamed(_) => Some(quote! {
            (#(#exprs,)*)
        }),
        Fields::Unit => None,
    };

    let variant_path = variant.map(|v| quote!(::#v));
    let from_fn = quote! {
        fn from(value: #tuple) -> Self {
            Self #variant_path #setter
        }
    };

    (tuple, from_fn)
}

fn do_impl_default(fields: &Fields, metadatas: &HashMap<Member, FieldMetadata>) -> TokenStream {
    fn default_of_field(field_member: &Member, default: &Option<Option<Expr>>) -> TokenStream {
        let (ident, colon) = match field_member {
            Member::Named(id) => (Some(id), Some(<Token![:]>::default())),
            Member::Unnamed(_) => (None, None),
        };

        let default_default_expr: Expr = parse_quote!(::std::default::Default::default());

        let default_behavior=match default{
            Some(Some(expr))=>expr,
            Some(None)=>&default_default_expr,
            None=>return Error::new(
                match ident{
                    Some(id)=>id.span(),
                    None=>Span::call_site()
                },
                format!(
                    "field `{}` has no default behavior, which is required due to `impl_default` attribute",
                    match field_member{
                        Member::Named(id)=>id.to_string(),
                        Member::Unnamed(Index { index, .. })=>index.to_string()
                    }
                )
            ).to_compile_error()
        };

        quote! {
            #ident #colon #default_behavior
        }
    }

    match fields {
        Fields::Unit => quote!(),
        Fields::Named(_) => {
            let iter = metadatas
                .iter()
                .map(|(member, FieldMetadata { default, .. })| default_of_field(member, default));
            quote! {
                {#(#iter),*}
            }
        }
        Fields::Unnamed(_) => {
            let mut metadata_sorted: Vec<(&Member, &FieldMetadata)> = metadatas.iter().collect();
            metadata_sorted.sort_by_key(|(m, _)| match m {
                Member::Named(_) => unreachable!(),
                Member::Unnamed(Index { index, .. }) => index,
            });
            let iter = metadata_sorted
                .iter()
                .map(|(member, FieldMetadata { default, .. })| default_of_field(member, default));
            quote! {
                (#(#iter),*)
            }
        }
    }
}
