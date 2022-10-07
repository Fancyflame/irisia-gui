use syn::{
    parse::{Nothing, Parse},
    punctuated::Punctuated,
    ItemFn,
};

/*
{
    fn field1(&self)->String{...}
    fn field2(&self)->u32{...}
}
*/
pub struct BracedItemFn(pub Vec<ItemFn>);

impl Parse for BracedItemFn {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        braced!(content in input);

        let punct: Punctuated<_, Nothing> = content.parse_terminated(ItemFn::parse)?;

        Ok(BracedItemFn(punct.into_iter().collect()))
    }
}
