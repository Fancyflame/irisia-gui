use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{braced, parse::Parse, parse_quote, Expr, Pat, Result, Token, Type};

pub struct ExprMatchData {
    expr: Expr,
    arms: Vec<Arm>,
}

pub struct Arm {
    pat: Pat,
    as_type: Option<Type>,
    key: Option<(Pat, Option<Type>)>,
    if_guard: Option<(Token![if], Expr)>,
    fat_arrow_token: Token![=>],
    body: Expr,
    comma: Option<Token![,]>,
}

impl Parse for ExprMatchData {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let expr = input.parse()?;
        input.parse::<Token![=>]>()?;
        let content;
        braced!(content in input);
        let mut arms = Vec::new();

        while !content.is_empty() {
            arms.push(content.parse()?);
        }

        Ok(ExprMatchData { expr, arms })
    }
}

impl Parse for Arm {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        Ok(Arm {
            pat: input.parse()?,
            as_type: {
                if input.peek(Token![as]) {
                    input.parse::<Token![as]>()?;
                    Some(input.parse()?)
                } else {
                    None
                }
            },
            key: {
                if input.peek(Token![,]) {
                    input.parse::<Token![,]>()?;
                    let pat = input.parse()?;
                    let as_type = if input.peek(Token![as]) {
                        input.parse::<Token![as]>()?;
                        Some(input.parse()?)
                    } else {
                        None
                    };
                    Some((pat, as_type))
                } else {
                    None
                }
            },
            if_guard: {
                if input.peek(Token![if]) {
                    Some((input.parse()?, input.parse()?))
                } else {
                    None
                }
            },
            fat_arrow_token: input.parse()?,
            body: input.parse()?,
            comma: input.parse()?,
        })
    }
}

pub fn expand(ExprMatchData { expr, arms }: ExprMatchData) -> TokenStream {
    expand_arm_reversed(
        quote!(((#expr) as cream_core::event::event_channel::data::Data<'_>)),
        arms.into_iter().rev(),
    )
}

fn expand_arm_reversed(expr: impl ToTokens, mut arms: impl Iterator<Item = Arm>) -> TokenStream {
    let Some(Arm {
        pat,
        key,
        as_type,
        if_guard,
        fat_arrow_token,
        body,
        comma,
    }) = arms.next() else {
        return quote!{()}
    };

    let as_type = as_type.unwrap_or_else(|| parse_quote!(_));

    let (match_expr, arm_pat) = match key {
        Some((key_pat, key_type)) => {
            let key_type = key_type.unwrap_or_else(|| parse_quote!(_));
            (
                quote! {#expr.assume_keyed::<#as_type, #key_type>()},
                quote! {(#pat, #key_pat)},
            )
        }
        None => (quote! {#expr.assume::<#as_type>()}, quote! {#pat}),
    };

    let if_guard = if_guard.map(|(if_token, cond)| quote! {#if_token #cond});

    let var: Ident = Ident::new("___cream_macro_match_data_variable", Span::call_site());

    let err_case = expand_arm_reversed(&var, arms);

    quote! {
        match #match_expr {
            ::std::result::Result::Ok(#arm_pat) #if_guard #fat_arrow_token #body #comma
            ::std::result::Result::Err(#var) => #err_case
        }
    }
}
