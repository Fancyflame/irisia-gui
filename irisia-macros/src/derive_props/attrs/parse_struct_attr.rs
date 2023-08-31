use proc_macro2::{Ident, TokenStream};
use quote::format_ident;
use syn::{
    parse::{ParseStream, Parser},
    punctuated::Punctuated,
    Error, LitStr, Result, Token, Visibility,
};

use super::StructAttr;

impl StructAttr {
    pub fn parse_from(attr: TokenStream, default_vis: Visibility) -> Result<Self> {
        (|input: ParseStream| parse(input, default_vis)).parse2(attr)
    }
}

fn parse(input: ParseStream, default_vis: Visibility) -> Result<StructAttr> {
    let mut vis: Option<Visibility> = None;
    let mut update_result: Option<Ident> = None;

    let updater_name: Ident = input.parse()?;

    if !input.is_empty() {
        input.parse::<Token![,]>()?;
    }

    let args: Punctuated<(Ident, LitStr), Token![,]> =
        Punctuated::parse_terminated_with(input, |stream| {
            Ok((stream.parse()?, {
                stream.parse::<Token![=]>()?;
                stream.parse()?
            }))
        })?;

    for (key, value) in args.into_iter() {
        match key.to_string().as_str() {
            "vis" => vis = Some(value.parse()?),
            "update_result" => update_result = Some(value.parse()?),
            _ => {
                return Err(Error::new_spanned(
                    &key,
                    format!("unrecognized key `{key}`"),
                ))
            }
        }
    }

    Ok(StructAttr {
        visibility: vis.unwrap_or(default_vis),
        update_result: update_result
            .unwrap_or_else(|| format_ident!("{}UpdateResult", updater_name)),
        updater_name,
    })
}
