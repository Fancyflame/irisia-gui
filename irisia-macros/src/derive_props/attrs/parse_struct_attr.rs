use proc_macro2::TokenStream;
use syn::{
    parse::{ParseStream, Parser},
    LitStr, Result, Token, Visibility,
};

use super::StructAttr;

impl StructAttr {
    pub fn parse_from(attr: TokenStream, default_vis: Visibility) -> Result<Self> {
        (|input: ParseStream| parse(input, default_vis)).parse2(attr)
    }
}

fn parse(input: ParseStream, default_vis: Visibility) -> Result<StructAttr> {
    Ok(StructAttr {
        updater_name: input.parse()?,
        visibility: if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let litstr: LitStr = input.parse()?;
            litstr.parse()?
        } else {
            default_vis
        },
    })
}
