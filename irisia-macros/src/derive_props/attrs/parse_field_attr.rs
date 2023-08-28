use proc_macro2::{Ident, Span};
use quote::ToTokens;
use syn::{braced, parse::ParseStream, Attribute, Error, ExprPath, MetaList, Result, Token, Type};

use super::{FieldAttr, FieldDefault, FieldOptions, FieldResolver};

impl FieldAttr {
    pub fn parse_from(attrs: &[Attribute], field_span: Span) -> Result<Self> {
        let mut builder = FieldAttrBuilder::Normal {
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

            if builder.is_skipped() {
                return Ok(FieldAttr::Skipped);
            }
        }

        builder.build(field_span)
    }
}

enum FieldAttrBuilder {
    Skipped,
    Normal {
        resolver: Option<FieldResolver>,
        default: Option<FieldDefault>,
        options: FieldOptions,
    },
}

impl FieldAttrBuilder {
    fn update(&mut self, path: &Ident, stream: ParseStream) -> Result<()> {
        let (resolver, default, options) = match self {
            Self::Skipped => return Ok(()),
            Self::Normal {
                resolver,
                default,
                options,
            } => (resolver, default, options),
        };

        let dup_resolver_err = || {
            Error::new_spanned(
                path,
                "resolver has already declared. \
        at most one of `moved`, `updater` and `with` can be declared",
            )
        };

        let dup_default_beh_err = || {
            Error::new_spanned(
                path,
                "default behavior has already declared. \
        only one of default behavior can and must be declared`",
            )
        };

        match path.to_string().as_str() {
            "skip" => {
                *self = Self::Skipped;
                return Ok(());
            }
            "moved" => set_option(resolver, FieldResolver::MoveOwnership, dup_resolver_err),
            "updater" => set_option(resolver, FieldResolver::CallUpdater, dup_resolver_err),
            "with" => set_option(resolver, parse_with_fn(stream)?, dup_resolver_err),
            "must_init" => set_option(default, FieldDefault::MustInit, dup_default_beh_err),
            "default" => set_option(default, FieldDefault::Default, dup_default_beh_err),
            "default_with" => set_option(default, parse_default_with(stream)?, dup_default_beh_err),
            _ => return Err(Error::new_spanned(path, format!("unknown key `{path}`"))),
        }
    }

    fn build(self, span: Span) -> Result<FieldAttr> {
        match self {
            Self::Skipped => Ok(FieldAttr::Skipped),
            Self::Normal {
                resolver,
                default,
                options,
            } => Ok(FieldAttr::Normal {
                value_resolver: resolver.unwrap_or(FieldResolver::MoveOwnership),
                default_behavior: match default {
                    Some(d) => d,
                    None => return Err(Error::new(span, "default behavior is required")),
                },
                options,
            }),
        }
    }

    fn is_skipped(&self) -> bool {
        matches!(self, Self::Skipped)
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

fn parse_default_with(stream: ParseStream) -> Result<FieldDefault> {
    let content;
    braced!(content in stream);
    let ep = content.parse::<ExprPath>()?;
    Ok(FieldDefault::DefaultWith(ep))
}

fn set_option<T>(option: &mut Option<T>, value: T, err: impl FnOnce() -> Error) -> Result<()> {
    if option.is_some() {
        return Err(err());
    }
    *option = Some(value);
    Ok(())
}
