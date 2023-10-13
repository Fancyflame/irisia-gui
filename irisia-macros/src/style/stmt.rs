use std::cell::{Ref, RefCell, RefMut};

use case::CaseExt;
use quote::{quote, ToTokens};
use syn::{
    parse::Parse, parse_quote, visit_mut::VisitMut, Error, Expr, ExprLit, Ident, Lit, Result,
    Token, Type, TypePath,
};

use crate::expr::StateExpr;

use super::StyleCodegen;

enum StyleType {
    Type(Type),
    Follow(Token![~]),
}

pub struct StyleStmt {
    style_ty: RefCell<StyleType>,
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
                style_ty: RefCell::new(style_ty),
                args: Vec::new(),
                options: Vec::new(),
            });
        }

        input.parse::<Token![:]>()?;

        let mut args = Vec::new();
        loop {
            if !input.peek(Token![.]) && !input.peek(Token![;]) {
                let mut arg = input.parse()?;
                special_lit(&mut arg);
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
            style_ty: RefCell::new(style_ty),
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
                special_lit(&mut e);
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

        let style_ty_ref = Ref::map(style_ty.borrow(), |t| match t {
            StyleType::Type(t) => t,
            StyleType::Follow(_) => {
                panic!("inner error: style follow not handled");
            }
        });

        let style_ty = &*style_ty_ref;

        let options = options.iter().map(|x| {
            let OptionArg { dot, name, expr } = x;
            match expr {
                Some(ex) => quote!(#dot call_func(<#style_ty>::#name, #ex)),
                None => quote!(#dot call_func_no_arg(<#style_ty>::#name)),
            }
        });

        quote! {
            irisia::style::Once(
                irisia::__private::new_chain_caller(
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

fn special_lit(expr: &mut Expr) {
    struct LitReplacer;

    impl VisitMut for LitReplacer {
        fn visit_expr_mut(&mut self, expr: &mut Expr) {
            if let Expr::Lit(ExprLit { lit, .. }) = expr {
                match lit {
                    Lit::Int(lit_int) if lit_int.suffix() == "px" => {
                        let val = lit_int.base10_parse::<u32>().unwrap() as f32;
                        *expr = parse_quote!(irisia::primitive::Pixel(#val));
                    }
                    Lit::Float(lit_float) if lit_float.suffix() == "px" => {
                        let val = lit_float.base10_parse::<f32>().unwrap();
                        *expr = parse_quote!(irisia::primitive::Pixel(#val));
                    }
                    Lit::Float(lit_float) if lit_float.suffix() == "pct" => {
                        let val = lit_float.base10_parse::<f32>().unwrap() / 100.0;
                        *expr = parse_quote!(#val);
                    }
                    _ => {}
                }
            } else {
                syn::visit_mut::visit_expr_mut(&mut LitReplacer, expr);
            }
        }
    }

    LitReplacer::visit_expr_mut(&mut LitReplacer, expr);
}

pub fn handle_style_follow(stmts: &[StateExpr<StyleCodegen>]) -> Result<()> {
    let mut prev_type: Option<RefMut<Type>> = None;
    for stmt in stmts {
        match stmt {
            StateExpr::Raw(r) => {
                let mut borrowed = r.style_ty.borrow_mut();

                match &mut *borrowed {
                    StyleType::Type(_) => {
                        prev_type = Some(RefMut::map(borrowed, |b| match b {
                            StyleType::Type(t) => t,
                            _ => unreachable!(),
                        }));
                    }

                    StyleType::Follow(f) => match &prev_type {
                        Some(t) => *borrowed = StyleType::Type((*t).clone()),
                        None => {
                            return Err(Error::new_spanned(
                                f,
                                "cannot infer the type that following",
                            ))
                        }
                    },
                }
            }

            StateExpr::Block(b) => handle_style_follow(&b.stmts)?,
            StateExpr::Command(_) => {}
            StateExpr::Conditional(c) => {
                for arm in c.arms() {
                    handle_style_follow(arm)?
                }
            }
            StateExpr::Repetitive(r) => handle_style_follow(&r.body().stmts)?,
        }
    }

    Ok(())
}
