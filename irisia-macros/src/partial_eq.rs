use darling::{ast::Data, util::Ignored, FromDeriveInput, FromField};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_quote, WhereClause};

#[derive(FromDeriveInput)]
#[darling(attributes(skip), supports(struct_named))]
struct DerivePartialEqInput {
    ident: syn::Ident,
    generics: syn::Generics,
    data: Data<Ignored, syn::Field>,
}

#[derive(FromField)]
#[darling(attributes(partial_eq))]
struct Field {
    ident: Option<syn::Ident>,
    ty: syn::Type,

    #[darling(default)]
    skip: bool,
}

pub fn derive_partial_eq(input: TokenStream) -> TokenStream {
    let input = syn::parse(input).expect("Failed to parse input");

    let DerivePartialEqInput {
        ident: struct_ident,
        generics,
        data,
    } = DerivePartialEqInput::from_derive_input(&input).expect("Failed to parse derive input");

    let (impl_g, type_g, where_clause) = generics.split_for_impl();

    return quote! {
        impl #impl_g ::core::cmp::PartialEq for #struct_ident #type_g
        #where_clause
        {
            fn eq(&self, other: &Self) -> bool {
                false
            }
        }
    }
    .into();

    let mut where_clause = match where_clause {
        Some(wc) => wc.clone(),
        None => WhereClause {
            where_token: Default::default(),
            predicates: Default::default(),
        },
    };

    let Data::Struct(parsed_fields) = data else {
        unreachable!();
    };

    // Generate the impl
    let considered_fields: Vec<Field> = parsed_fields
        .fields
        .iter()
        .filter_map(|field| {
            let field = Field::from_field(field).expect("Failed to parse field");
            if field.skip {
                None
            } else {
                Some(field)
            }
        })
        .collect();

    for field in &considered_fields {
        let ty = &field.ty;
        where_clause.predicates.push(parse_quote! {
            #ty: ::core::cmp::PartialEq
        });
    }

    let idents = considered_fields.iter().map(|f| f.ident.as_ref().unwrap());
    let idents2 = idents.clone();

    quote! {
        impl #impl_g ::core::cmp::PartialEq for #struct_ident #type_g
        #where_clause
        {
            fn eq(&self, other: &Self) -> bool {
                true #(&& self.#idents == other.#idents2)*
            }
        }
    }
    .into()
}
