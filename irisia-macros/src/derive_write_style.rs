use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{spanned::Spanned, Data, DeriveInput, Error, Index, Member, Result};

pub fn derive(
    DeriveInput {
        ident: struct_name,
        generics,
        data,
        ..
    }: DeriveInput,
) -> Result<TokenStream> {
    let fields = match data {
        Data::Struct(data) => data.fields,
        _ => return Err(Error::new(Span::call_site(), "only struct is support")),
    };

    let members = fields.iter().enumerate().map(|(index, f)| match &f.ident {
        Some(id) => Member::Named(id.clone()),
        None => Member::Unnamed(Index {
            index: index as _,
            span: f.ty.span(),
        }),
    });

    let (impl_gen, type_gen, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_gen ::irisia::style::WriteStyle for #struct_name #type_gen
        #where_clause
        {
            fn write_style<T>(&mut self, _read: &T)
            where
                T: ::irisia::style::ReadStyle + ?::std::marker::Sized,
            {
                #(::irisia::style::WriteStyle::write_style(
                    &mut self.#members,
                    _read
                );)*
            }
        }
    })
}
