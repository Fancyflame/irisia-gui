use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, ToTokens};
use syn::{parse_quote, spanned::Spanned, visit_mut::VisitMut, Expr, ExprLit, Lit};

pub fn handle_expr(mut i: Expr) -> HandledExpr {
    Visitor.visit_expr_mut(&mut i);
    HandledExpr(i)
}

struct Visitor;

impl VisitMut for Visitor {
    fn visit_expr_mut(&mut self, i: &mut Expr) {
        let (value_result, suffix) = if let Expr::Lit(ExprLit { ref lit, .. }) = i {
            match lit {
                Lit::Float(f) => (f.base10_parse::<f32>(), f.suffix()),
                Lit::Int(int) => (int.base10_parse::<i32>().map(|i| i as f32), int.suffix()),
                _ => return,
            }
        } else {
            syn::visit_mut::visit_expr_mut(self, i);
            return;
        };

        let value = match value_result {
            Ok(value) => {
                let mut lit = Literal::f32_suffixed(value);
                lit.set_span(i.span());
                lit
            }
            Err(_) => return,
        };

        let method = match suffix {
            "px" | "vw" | "vh" | "vmin" | "vmax" | "pw" | "ph" | "pmin" | "pmax" => {
                format_ident!("{suffix}", span = i.span())
            }
            _ => return,
        };

        let new_expr: Expr = parse_quote! {
            (::irisia::primitive::Length::#method(#value))
        };

        *i = new_expr;
    }
}

pub struct HandledExpr(Expr);

impl ToTokens for HandledExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}
