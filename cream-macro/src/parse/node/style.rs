use proc_macro2::Span;
use std::collections::HashMap;
use syn::{
    parse::{Nothing, Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Expr, Ident, Lit, Token, TypeInfer, TypePath,
};

pub struct Style(pub Vec<StyleItem>);

impl Parse for Style {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        braced!(content in input);
        let punct: Punctuated<_, Nothing> = content.parse_terminated(StyleItem::parse)?;
        Ok(Style(punct.into_iter().collect()))
    }
}

// ::path::to::Style:
//   10 10 1fr
//   {self.a+=10;a+1}
//   option1=20
//   option2=10
// ;
pub struct StyleItem {
    pub style: StyleType,
    pub exprs: Vec<ExprShortCut>,
    pub options: HashMap<Ident, ExprShortCut>,
}

impl Parse for StyleItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let style: StyleType = input.parse()?;
        input.parse::<Token![:]>()?;
        let mut exprs = Vec::new();
        let mut options: HashMap<Ident, ExprShortCut> = HashMap::new();

        loop {
            if input.peek(Token![;]) {
                input.parse::<Token![;]>()?;
                break;
            } else if input.peek(Ident) && input.peek2(Token![=]) {
                options.insert(input.parse()?, {
                    input.parse::<Token![=]>()?;
                    input.parse()?
                });
            } else {
                if options.len() != 0 {
                    return Err(
                        input.error("Cannot declare arguments after declaration of optional field")
                    );
                }
                exprs.push(input.parse()?);
            }
        }

        Ok(StyleItem {
            style,
            exprs,
            options,
        })
    }
}

pub enum StyleType {
    Specified(TypePath),
    Auto(TypeInfer),
}

impl StyleType {
    pub fn span(&self) -> Span {
        match self {
            StyleType::Specified(tp) => tp.span(),
            StyleType::Auto(ti) => ti.span(),
        }
    }
}

impl Parse for StyleType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(if input.peek(Token![_]) {
            StyleType::Auto(input.parse()?)
        } else {
            StyleType::Specified(TypePath {
                qself: None,
                path: input.parse()?,
            })
        })
    }
}

pub struct ExprShortCut {
    pub span: Span,
    pub value: ShortCutValue,
}

pub enum ShortCutValue {
    Frame(u32),
    Pixel(i32),
    Percent(f32),
    Expr(Expr),
}

impl Parse for ExprShortCut {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let expr: Expr = input.parse()?;
        let span = expr.span();
        let value = if let Expr::Lit(ref el) = expr {
            match &el.lit {
                Lit::Int(li) if li.suffix() == "px" => ShortCutValue::Pixel(li.base10_parse()?),
                Lit::Int(li) if li.suffix() == "fr" => ShortCutValue::Frame(li.base10_parse()?),
                Lit::Float(lf) if lf.suffix() == "pct" => {
                    ShortCutValue::Percent(lf.base10_parse()?)
                }
                _ => ShortCutValue::Expr(expr),
            }
        } else {
            ShortCutValue::Expr(expr)
        };
        Ok(ExprShortCut { span, value })
    }
}
