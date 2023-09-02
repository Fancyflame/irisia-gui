use std::collections::HashSet;

use proc_macro2::{Ident, TokenStream};
use quote::format_ident;
use syn::{
    meta::ParseNestedMeta, parse::ParseStream, punctuated::Punctuated, spanned::Spanned,
    token::Paren, Error, LitStr, Result, Token, Visibility,
};

use super::{DefaultWatch, StructAttr};

impl StructAttr {
    pub fn parse_from(attr: TokenStream, default_vis: Visibility) -> Result<Self> {
        let mut vis: Option<Visibility> = None;
        let mut update_result: Option<Ident> = None;
        let mut default_watch: Option<DefaultWatch> = None;
        let mut updater_name: Option<Ident> = None;

        syn::meta::parser(|nested| {
            let ident = nested
                .path
                .get_ident()
                .ok_or_else(|| Error::new_spanned(&nested.path, "expected identifier"))?;
            macro_rules! get_str {
                () => {
                    nested.value()?.parse::<LitStr>()?.parse()?
                };
            }

            match ident.to_string().as_str() {
                "updater" => updater_name = Some(get_str!()),
                "vis" => vis = Some(get_str!()),
                "update_result" => update_result = Some(get_str!()),
                "watch" => default_watch = Some(parse_watch(nested)?),
                _ => {
                    return Err(Error::new_spanned(
                        &ident,
                        format!("unrecognized key `{ident}`"),
                    ))
                }
            }

            Ok(())
        });

        let updater_name = updater_name
            .ok_or_else(|| Error::new(attr.span(), "option `updater` must be provided"))?;

        Ok(StructAttr {
            visibility: vis.unwrap_or(default_vis),
            update_result: update_result
                .unwrap_or_else(|| format_ident!("{}UpdateResult", updater_name)),
            updater_name,
            default_watch,
        })
    }
}

fn parse_watch(nested: ParseNestedMeta) -> Result<DefaultWatch> {
    let mut default_watch = DefaultWatch {
        group_name: format_ident!("changed"),
        exclude: HashSet::new(),
    };

    if !nested.input.peek(Paren) {
        return Ok(default_watch);
    }

    nested.parse_nested_meta(|nested2| {
        let ident = nested2
            .path
            .get_ident()
            .ok_or_else(|| Error::new_spanned(&nested2.path, "expected identifier"))?;

        match ident.to_string().as_str() {
            "group" => default_watch.group_name = nested2.value()?.parse::<LitStr>()?.parse()?,
            "exclude" => insert_exclude(&mut default_watch.exclude, nested2.input)?,
            other => {
                return Err(Error::new_spanned(
                    ident,
                    format!("unrecognized option `{other}`"),
                ))
            }
        }
        Ok(())
    })?;

    Ok(default_watch)
}

fn insert_exclude(set: &mut HashSet<Ident>, input: ParseStream) -> Result<()> {
    let groups = input
        .parse::<LitStr>()?
        .parse_with(Punctuated::<Ident, Token![,]>::parse_terminated)?;

    for group in groups.into_iter() {
        if set.contains(&group) {
            return Err(Error::new_spanned(
                &group,
                format!("excluded field `{group}` has declared"),
            ));
        }

        set.insert(group);
    }

    Ok(())
}
