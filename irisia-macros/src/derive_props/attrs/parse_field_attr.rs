use proc_macro2::{Ident, Span};
use syn::{braced, parse::ParseStream, Attribute, Error, ExprPath, LitStr, Result, Token, Type};

use super::{FieldAttr, FieldDefault, FieldOptions, FieldResolver};

impl FieldAttr {
    pub fn parse_from(attrs: &[Attribute], field_span: Span) -> Result<Self> {
        let mut builder = FieldAttrBuilder {
            resolver: None,
            default: None,
            options: FieldOptions { rename: None },
        };

        for attr in attrs {
            if !attr.path().is_ident("props") {
                continue;
            }

            attr.meta.require_list()?.parse_nested_meta(|nested| {
                let Some(ident) = nested.path.get_ident()
                else {
                    return Err(Error::new_spanned(&nested.path, "expected ident"));
                };

                builder.update(ident, nested.input)
            })?;
        }

        builder.build(field_span)
    }
}

struct FieldAttrBuilder {
    resolver: Option<FieldResolver>,
    default: Option<FieldDefault>,
    options: FieldOptions,
}

impl FieldAttrBuilder {
    fn update(&mut self, path: &Ident, stream: ParseStream) -> Result<()> {
        let FieldAttrBuilder {
            resolver,
            default,
            options,
        } = self;

        let dup_resolver_err = || {
            Error::new_spanned(
                path,
                "resolver has already declared. at most one of \
                `moved`, `updater`, `with` and `resolver` can be declared",
            )
        };

        let dup_default_beh_err = || {
            Error::new_spanned(
                path,
                "default behavior has already declared. \
                only one of default behavior can and must be declared`",
            )
        };

        let dup_option = |option: &str| {
            Error::new_spanned(path, format!("option `{option}` has already declared."))
        };

        match path.to_string().as_str() {
            "moved" => set_option(resolver, FieldResolver::MoveOwnership, dup_resolver_err),
            "updated" => set_option(resolver, FieldResolver::CallUpdater, dup_resolver_err),
            "read_style" => set_option(resolver, FieldResolver::ReadStyle, dup_resolver_err),
            "with" => set_option(resolver, parse_with_fn(stream)?, dup_resolver_err),
            "resolver" => set_option(resolver, parse_custom_resolver(stream)?, dup_resolver_err),
            "must_init" => set_option(default, FieldDefault::MustInit, dup_default_beh_err),
            "default" => set_option(default, parse_default(stream)?, dup_default_beh_err),
            "rename" => set_option(&mut options.rename, parse_rename(stream)?, || {
                dup_option("rename")
            }),
            _ => return Err(Error::new_spanned(path, format!("unknown option `{path}`"))),
        }
    }

    fn build(self, span: Span) -> Result<FieldAttr> {
        let FieldAttrBuilder {
            resolver,
            default,
            options,
        } = self;

        Ok(FieldAttr {
            value_resolver: resolver.unwrap_or(FieldResolver::MoveOwnership),
            default_behavior: match default {
                Some(d) => d,
                None => return Err(Error::new(span, "default behavior is required")),
            },
            options,
        })
    }
}

fn parse_with_fn(stream: ParseStream) -> Result<FieldResolver> {
    let content;
    braced!(content in stream);
    let ep = content.parse::<ExprPath>()?;
    content.parse::<Token![,]>()?;
    let ty = content.parse::<Type>()?;
    Ok(FieldResolver::WithFn {
        path: ep,
        arg_type: ty,
    })
}

fn parse_custom_resolver(stream: ParseStream) -> Result<FieldResolver> {
    let content;
    braced!(content in stream);
    Ok(FieldResolver::Custom(content.parse()?))
}

fn parse_default(stream: ParseStream) -> Result<FieldDefault> {
    if stream.peek(Token![=]) {
        stream.parse::<Token![=]>()?;
        Ok(FieldDefault::DefaultWith(
            stream.parse::<LitStr>()?.parse()?,
        ))
    } else {
        Ok(FieldDefault::Default)
    }
}

fn parse_rename(stream: ParseStream) -> Result<Ident> {
    stream.parse::<Token![=]>()?;
    stream.parse::<LitStr>()?.parse()
}

fn set_option<T>(option: &mut Option<T>, value: T, err: impl FnOnce() -> Error) -> Result<()> {
    if option.is_some() {
        return Err(err());
    }
    *option = Some(value);
    Ok(())
}