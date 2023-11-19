use std::collections::{hash_map::Entry, HashMap};

pub(super) use parse_child::ElementStmt;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Error, Expr, Ident, Result, Token,
};

use super::{App, DataDec, PropDec};

mod parse_child;

impl Parse for App {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut output = leading_app(input)?;

        while !input.is_empty() {
            input.parse::<Token![,]>()?;

            let item: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            match &*item.to_string() {
                "app" => {
                    return Err(Error::new(
                        item.span(),
                        "one app can only have one signature",
                    ))
                }

                "data" => parse_data(input, &mut output.data)?,
                "props" => parse_props(input, &mut output.props)?,
                "watch" => parse_watch(input, &mut output.watch)?,

                "child" => {
                    if output.child_node.replace(input.parse()?).is_some() {
                        return Err(Error::new(
                            item.span(),
                            "duplicated child declaration found, one app can only have one child",
                        ));
                    }
                }

                "mount" => {
                    if output.on_mount.replace(input.parse()?).is_some() {
                        return Err(Error::new(
                            item.span(),
                            "duplicated mount hook found. try merge them into one",
                        ));
                    }
                }

                other => {
                    return Err(Error::new(
                        other.span(),
                        format!("unknown item declared: {other}"),
                    ))
                }
            }
        }

        Ok(output)
    }
}

fn braced_list<F>(input: ParseStream, mut f: F) -> Result<()>
where
    F: FnMut(ParseStream) -> Result<()>,
{
    let content;
    braced!(content in input);

    while !content.is_empty() {
        f(&content)?;
        if !content.is_empty() {
            content.parse::<Token![,]>()?;
        }
    }

    Ok(())
}

fn leading_app(input: ParseStream) -> Result<App> {
    let item: Ident = input.parse()?;
    if item != "app" {
        return Err(Error::new(item.span(), "the first item must be `app`"));
    }
    input.parse::<Token![:]>()?;

    Ok(App {
        vis: input.parse()?,
        ident: input.parse()?,
        data: HashMap::new(),
        props: HashMap::new(),
        watch: HashMap::new(),
        child_node: None,
        on_mount: None,
    })
}

fn parse_data(input: ParseStream, data: &mut HashMap<Ident, DataDec>) -> Result<()> {
    braced_list(input, |content| match data.entry(content.parse()?) {
        Entry::Occupied(occ) => Err(Error::new(
            occ.key().span(),
            format!("cannot declare data `{}` twice", occ.key()),
        )),
        Entry::Vacant(vac) => {
            vac.insert(DataDec {
                ty: {
                    content.parse::<Token![:]>()?;
                    content.parse()?
                },
                init: {
                    content.parse::<Token![=]>()?;
                    content.parse()?
                },
            });
            Ok(())
        }
    })
}

fn parse_props(input: ParseStream, props: &mut HashMap<Ident, PropDec>) -> Result<()> {
    braced_list(input, |content| match props.entry(content.parse()?) {
        Entry::Occupied(occ) => Err(Error::new(
            occ.key().span(),
            format!("cannot declare property `{}` twice", occ.key()),
        )),
        Entry::Vacant(vac) => {
            vac.insert(PropDec {
                ty: {
                    content.parse::<Token![:]>()?;
                    content.parse()?
                },
                default: if content.peek(Token![=]) {
                    content.parse::<Token![=]>()?;
                    Some(content.parse()?)
                } else {
                    None
                },
            });
            Ok(())
        }
    })
}

fn parse_watch(input: ParseStream, watch: &mut HashMap<Ident, Expr>) -> Result<()> {
    braced_list(input, |content| match watch.entry(content.parse()?) {
        Entry::Vacant(vac) => {
            content.parse::<Token![:]>()?;
            vac.insert(content.parse()?);
            Ok(())
        }
        Entry::Occupied(occ) => Err(Error::new(
            occ.key().span(),
            format!(
                "duplicated watcher of `{}` found. try merge them into one",
                occ.key()
            ),
        )),
    })
}
