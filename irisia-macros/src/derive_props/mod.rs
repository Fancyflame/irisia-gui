use proc_macro2::{TokenStream};
use quote::{format_ident, quote};
use syn::{
    punctuated::Punctuated, token::Comma, Field, Fields, Ident, ItemStruct,
    Result,
};
// attr详情<https://doc.rust-lang.org/stable/book/ch19-06-macros.html#attribute-like-macros>
pub fn props(_attr: TokenStream, item: ItemStruct) -> Result<TokenStream> {
    let fields = match &item.fields {
        Fields::Named(fields) => &fields.named,
        _ => panic!("expected a struct with named fields"),
    };
    let iter = (1..=fields.len()).map(|num| format_ident!("T{}", num));
    let (field_name, field_ty) = (
        fields.iter().map(|field| &field.ident),
        fields.iter().map(|field| &field.ty),
    );
    let (field_name_r, field_ty_r) = (field_name.clone(), field_ty.clone());
    let iter_impl = iter.clone();
    let iter_sign = iter.clone();
    let struct_gen = quote! {
        struct Foo<#(#iter=()),*>{
            #(
                #field_name_r:#field_ty_r,
            )*
        }

    };
    let gen_field = generate_field(fields)?;
    
    let impl_default = quote! {
        impl Default for Foo{
            fn default(){
            #(
                let #field_name:#field_ty = Default::default();
            )*
            }
        }
        impl <#(#iter_impl),*>Foo<#(#iter_sign),*>{
            #gen_field
        }
    };

    let gen = quote! {
        #struct_gen
        #impl_default
    };
    Ok(gen)
}
fn generate_field(fields: &Punctuated<Field, Comma>) -> Result<TokenStream> {
    let iter = (1..=fields.len()).map(|num| format_ident!("T{}", num));
    let iter_vec: Vec<Ident> = iter.clone().collect();
    let mut gen_vec = vec![];
    let return_generic = quote! {
        Foo<#(#iter),*>
    };
    for (index_o, fn_name) in fields.iter().enumerate() {
        let iter_index = &iter_vec[index_o];
        for (_index, ident) in fn_name.ident.iter().enumerate() {
            if fn_name.ident.as_ref() == Some(ident) {
                gen_vec.push(quote! {
                  pub fn #ident<#iter_index> (self,value:#iter_index)->#return_generic{
                    Foo{
                        #ident:(value,),
                    }
                  }
                });
            } else {
                gen_vec.push(quote! {
                  pub fn #ident<#iter_index> (self,value:#iter_index)->#return_generic{
                    Foo{
                        #ident:self.#ident,
                    }
                  }
                })
            }
        }
    }
    let gen_iter =gen_vec.into_iter();
    let gen = quote!{
        #(#gen_iter)*
    };
    Ok(gen)
}
