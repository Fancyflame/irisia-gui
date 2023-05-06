use case::CaseExt;
use quote::{quote, ToTokens};
use syn::{
    parse::Parse, parse_quote, Error, Expr, ExprLit, Ident, Lit, Result, Token, Type, TypePath,
};

use crate::expr::{StateExpr, VisitUnit};

use super::StyleCodegen;

enum StyleType {
    Type(Type),
    Follow(Token![~]),
}

pub struct StyleStmt {
    style_ty: StyleType,
    args: Vec<Expr>,
    options: Vec<OptionArg>,
}

struct OptionArg {
    dot: Token![.],
    name: Ident,
    expr: Option<Expr>,
}

impl Parse for StyleStmt {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut style_ty: StyleType = input.parse()?;

        if let StyleType::Type(Type::Path(TypePath { qself: None, path })) = &mut style_ty {
            if let Some(id) = path.get_ident() {
                let id_option = type_name_snake_to_camel(id);
                let id = id_option.as_ref().unwrap_or(id);
                *path = parse_quote!(#id);
            }
        }

        if input.peek(Token![;]) {
            input.parse::<Token![;]>()?;
            return Ok(StyleStmt {
                style_ty,
                args: Vec::new(),
                options: Vec::new(),
            });
        }

        input.parse::<Token![:]>()?;

        let mut args = Vec::new();
        loop {
            if !input.peek(Token![.]) && !input.peek(Token![;]) {
                let mut arg = input.parse()?;
                special_lit(&mut arg)?;
                args.push(arg);
                if input.peek(Token![,]) {
                    input.parse::<Token![,]>()?;
                    continue;
                }
            }
            break;
        }

        let mut options = Vec::new();
        loop {
            if input.peek(Token![.]) {
                options.push(input.parse()?);
                if input.peek(Token![,]) {
                    input.parse::<Token![,]>()?;
                    continue;
                }
            }
            break;
        }

        input.parse::<Token![;]>()?;

        Ok(StyleStmt {
            style_ty,
            args,
            options,
        })
    }
}

impl Parse for StyleType {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        if input.peek(Token![~]) {
            Ok(StyleType::Follow(input.parse::<Token![~]>()?))
        } else {
            Ok(StyleType::Type(input.parse()?))
        }
    }
}

impl Parse for OptionArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let arg = OptionArg {
            dot: input.parse()?,
            name: input.parse()?,
            expr: if input.peek(Token![,]) || input.peek(Token![;]) {
                None
            } else {
                let mut e = input.parse()?;
                special_lit(&mut e)?;
                Some(e)
            },
        };
        Ok(arg)
    }
}

fn type_name_snake_to_camel(id: &Ident) -> Option<Ident> {
    let s = id.to_string();

    let mut is_snake = true;
    for c in s.chars() {
        if !c.is_ascii() || c.is_ascii_uppercase() {
            is_snake = false;
            break;
        }
    }

    if is_snake {
        Some(Ident::new(&format!("Style{}", s.to_camel()), id.span()))
    } else {
        None
    }
}

impl ToTokens for StyleStmt {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let StyleStmt {
            style_ty,
            args,
            options,
        } = self;

        let style_ty = match style_ty {
            StyleType::Type(t) => t,
            StyleType::Follow(_) => {
                panic!("inner error: style follow not handled");
            }
        };

        let options = options.iter().map(|x| {
            let OptionArg { dot, name, expr } = x;
            match expr {
                Some(ex) => quote!(#dot call_func(<#style_ty>::#name, #ex)),
                None => quote!(#dot call_func_no_arg(<#style_ty>::#name)),
            }
        });

        quote! {
            irisia::style::AddStyle::new(
                irisia::__new_chain_caller(
                    <#style_ty as ::std::convert::From<_>>::from(
                        (#(#args,)*)
                    )
                )
                #(#options)*
                .finish()
            )
        }
        .to_tokens(tokens);
    }
}

fn special_lit(expr: &mut Expr) -> Result<()> {
    if let Expr::Lit(ExprLit { lit, .. }) = expr {
        match lit {
            Lit::Int(lit_int) if lit_int.suffix() == "px" => {
                let val = lit_int.base10_parse::<u32>()? as f32;
                *expr = parse_quote!(irisia::style::Pixel(#val));
            }
            Lit::Float(lit_float) if lit_float.suffix() == "px" => {
                let val = lit_float.base10_parse::<f32>()?;
                *expr = parse_quote!(irisia::style::Pixel(#val));
            }
            Lit::Float(lit_float) if lit_float.suffix() == "pct" => {
                let val = lit_float.base10_parse::<f32>()? / 100.0;
                *expr = parse_quote!(#val);
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn handle_style_follow(stmts: &mut [StateExpr<StyleCodegen>]) -> Result<()> {
    let mut type_stack: Vec<Option<Type>> = Vec::new();

    for stmt in stmts {
        stmt.visit_unit_mut(0, &mut |expr, depth| {
            let raw = match expr {
                StateExpr::Raw(r) => r,
                StateExpr::Command(_) => return Ok(()),
                _ => unreachable!(),
            };

            type_stack.resize(depth + 1, {
                const NONE_TYPE: Option<Type> = None;
                NONE_TYPE
            });

            match &raw.style_ty {
                StyleType::Type(t) => {
                    *type_stack.get_mut(depth).unwrap() = Some(t.clone());
                }
                StyleType::Follow(follow) => match &type_stack.get(depth).unwrap() {
                    Some(t) => {
                        raw.style_ty = StyleType::Type(t.clone());
                    }
                    None => {
                        return Err(Error::new_spanned(
                            follow,
                            "cannot infer the type that following",
                        ))
                    }
                },
            }

            return Ok(());
        })?;
    }

    Ok(())
}
