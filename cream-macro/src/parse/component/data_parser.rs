use std::collections::HashMap;

use proc_macro2::Ident;
use syn::{
    parse::Parse,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Brace, Paren},
    Expr, Type, Visibility,
};

pub struct Field {
    vis: Visibility,
    name: Ident,
    ty: Type,
    default: Option<Expr>,
}

impl Parse for Field {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let vis = input.parse()?;
        let name = input.parse()?;
        input.parse::<Token!(:)>()?;
        let ty = input.parse()?;

        let default = if input.peek(Token!(=)) {
            input.parse::<Token!(=)>()?;
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Field {
            vis,
            name,
            ty,
            default,
        })
    }
}

#[derive(Default)]
pub struct DataSection {
    parenthesis_arg: Vec<Ident>,
    fields: HashMap<Ident, Field>,
}

impl Parse for DataSection {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // data: (arg1: u32, arg2: u32){
        //     data1: String = "hola".to_string(),
        //     pub not_optional: usize,
        //     data2: Vec<u8> = vec![1, 2, 3, 4]
        // }

        let mut declared_at_least_one = false;
        let mut field_map: HashMap<Ident, Field> = HashMap::new();

        let mut insert_field = |field: Field| {
            let id = field.name.clone();

            if field_map.contains_key(&id) {
                parse_err!(
                    id.span(),
                    format!("Repeated data item: `{}`", id.to_string())
                );
            } else {
                field_map.insert(id, field);
            }

            Ok(())
        };

        // 圆括号参数列表
        let arg_list: Vec<Ident> = if input.peek(Paren) {
            declared_at_least_one = true;
            let content;
            parenthesized!(content in input);
            let punct: Punctuated<_, Token![,]> = content.parse_terminated(Field::parse)?;

            let mut vec = Vec::with_capacity(punct.len());
            for x in punct.into_iter() {
                // 不允许在圆括号参数列表里使用默认值
                if let Some(def) = x.default {
                    parse_err!(
                        def.span(),
                        "Cannot declare default expression in arguments list"
                    );
                }

                let id = x.name.clone();
                insert_field(x)?;
                vec.push(id);
            }
            vec
        } else {
            Vec::new()
        };

        // 花括号参数列表
        if input.peek(Brace) {
            declared_at_least_one = true;
            let content;
            braced!(content in input);

            loop {
                if content.is_empty() {
                    break;
                }

                insert_field(input.parse()?)?;

                if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                } else {
                    break;
                }
            }
        }

        if !declared_at_least_one {
            return Err(input
                .error("`data` has no tokens. Delete the item it if it is expected to be empty."));
        }

        Ok(DataSection {
            parenthesis_arg: arg_list,
            fields: field_map,
        })
    }
}
