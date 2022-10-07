use std::collections::HashSet;

use syn::{
    braced,
    parse::{Parse, ParseStream},
    Field, FieldsNamed, Ident, ItemFn, Result, Token, Visibility,
};

use self::{braced_item_fn::BracedItemFn, data_parser::DataSection};

use super::node::Node;

pub mod braced_item_fn;
pub mod data_parser;

pub struct Component {
    pub visibility: Visibility,
    pub name: Ident,
    pub data: DataSection,
    pub fields: Vec<Field>,
    pub computed: Vec<ItemFn>,
    pub watch: Vec<ItemFn>,
    pub methods: Vec<ItemFn>,
    pub body: Node,
}

impl Parse for Component {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // 保证每个根选项只声明一次
        let mut once_guard = HashSet::new();

        let mut visibility = Visibility::Inherited;
        let mut name = None;
        let mut data = None;
        let mut fields = None;
        let mut computed = Vec::new();
        let mut watch = Vec::new();
        let mut methods = Vec::new();
        let mut body = None;

        // 将Option转换成Result
        fn required<T>(name: &str, option: Option<T>) -> Result<T> {
            match option {
                Some(n) => Ok(n),
                None => Err(syn::Error::new_spanned(
                    "",
                    format!("Item `{name}` is required"),
                )),
            }
        }

        loop {
            if input.is_empty() {
                break;
            }

            let id: Ident = input.parse()?;
            let id_str = id.to_string();
            if !once_guard.insert(id.clone()) {
                parse_err!(
                    id.span(),
                    format!("Duplicated declaration of component option `{}`", id_str)
                );
            }
            input.parse::<Token![:]>()?;

            match &*id_str {
                "component" => {
                    // component: pub StructName
                    visibility = input.parse()?;
                    name = Some(input.parse()?);
                }
                "data" => {
                    // data: (arg1: u32, arg2: u32){
                    //     data1: String = "hola".to_string(),
                    //     not_optional: usize,
                    //     data2: Vec<u8> = vec![1, 2, 3, 4]
                    // }
                    data = Some(input.parse()?);
                }
                "fields" => {
                    let f: FieldsNamed = input.parse()?;
                    fields = Some(f.named.into_iter().collect());
                }
                "computed" => {
                    computed = input.parse::<BracedItemFn>()?.0;
                }
                "watch" => {
                    watch = input.parse::<BracedItemFn>()?.0;
                }
                "methods" => {
                    methods = input.parse::<BracedItemFn>()?.0;
                }
                "body" => {
                    // body: {
                    //     Root{
                    //         Node1{...}
                    //         Node2{...}
                    //     }
                    // }
                    let content;
                    braced!(content in input);
                    if content.is_empty() {
                        parse_err!(id.span(), "A component must has at least one node");
                    }
                    body = Some(content.parse()?);
                }
                _ => {
                    parse_err!(
                        id.span(),
                        "Unrecognized option at component declaration: `{}`",
                        id_str
                    );
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            } else {
                break;
            }
        }

        Ok(Component {
            visibility,
            name: required("component", name)?,
            data: data.unwrap_or_default(),
            fields: fields.unwrap_or_default(),
            computed,
            watch,
            methods,
            body: required("body", body)?,
        })
    }
}
