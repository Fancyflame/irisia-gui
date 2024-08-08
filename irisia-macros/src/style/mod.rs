use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    token::Brace,
    Expr, Ident, Pat, Path, Result, Token, Type,
};

use self::number_suffix::{handle_expr, HandledExpr};

mod number_suffix;

#[derive(Clone)]
pub enum BlockPath {
    Path(Path),
    Clear,
    Inherit,
}

pub struct Content {
    pub path: BlockPath,
    pub rules: Vec<StyleExpr>,
}

impl Content {
    pub fn only_rules(rules: Vec<StyleExpr>) -> Self {
        Self {
            path: BlockPath::Inherit,
            rules,
        }
    }

    pub fn generate_style_content(&self, path: Option<&Path>) -> TokenStream {
        let path = match &self.path {
            BlockPath::Path(path) => Some(path),
            BlockPath::Clear => None,
            BlockPath::Inherit => path,
        };

        self.rules
            .iter()
            .map(|rule| rule.generate_style_content(path))
            .collect()
    }
}

pub enum StyleExpr {
    Rule {
        ty: Type,
        args: Vec<HandledExpr>,
    },
    Content(Content),
    If {
        cond: HandledExpr,
        then: Content,
        else_branch: Box<StyleExpr>,
    },
    Match {
        expr: HandledExpr,
        arms: Vec<(Pat, Option<HandledExpr>, StyleExpr)>,
    },
    Let {
        name: Ident,
        expr: Box<StyleExpr>,
    },
    UseVar {
        value: HandledExpr,
    },
}

impl Parse for StyleExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        Self::parse_full(input, false)
    }
}

impl StyleExpr {
    fn parse_full(input: ParseStream, in_block: bool) -> Result<Self> {
        if input.peek(Token![in]) {
            Self::parse_path_block(input)
        } else if input.peek(Token![if]) {
            Self::parse_if_else(input)
        } else if input.peek(Token![match]) {
            Self::parse_match(input)
        } else if input.peek(Brace) {
            Self::parse_block(input).map(Self::Content)
        } else if input.peek(Token![use]) {
            Self::parse_use_var(input, in_block)
        } else if in_block && input.peek(Token![let]) {
            Self::parse_let(input)
        } else if in_block {
            Self::parse_single_rule(input)
        } else {
            Err(input.error("unexpect token, expect `in`, `if`, `match`, `let`, `use` or `{`"))
        }
    }

    fn parse_single_rule(input: ParseStream) -> Result<Self> {
        let ty: Type = input.parse()?;
        let mut args: Vec<HandledExpr> = Vec::new();

        if input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            loop {
                args.push(handle_expr(input.parse()?));
                if input.peek(Token![;]) {
                    break;
                }
                input.parse::<Token![,]>()?;
            }
        }

        input.parse::<Token![;]>()?;
        Ok(Self::Rule { ty, args })
    }

    fn parse_block_content(input: ParseStream, brace: bool) -> Result<Vec<StyleExpr>> {
        let mut _content = None;
        let content: ParseStream = if brace {
            let content;
            syn::braced!(content in input);
            _content.insert(content)
        } else {
            input
        };

        let mut rules = Vec::new();
        while !content.is_empty() {
            rules.push(Self::parse_full(content, true)?);
        }
        Ok(rules)
    }

    fn parse_let(input: ParseStream) -> Result<Self> {
        input.parse::<Token![let]>()?;
        let name: Ident = input.parse()?;
        input.parse::<Token![=]>()?;
        let expr: StyleExpr = input.parse()?;
        input.parse::<Token![;]>()?;

        Ok(Self::Let {
            name,
            expr: Box::new(expr),
        })
    }

    fn parse_use_var(input: ParseStream, in_block: bool) -> Result<Self> {
        input.parse::<Token![use]>()?;
        let value = handle_expr(input.parse()?);
        if in_block {
            input.parse::<Token![;]>()?;
        }
        Ok(Self::UseVar { value })
    }

    fn parse_block(input: ParseStream) -> Result<Content> {
        let rules = Self::parse_block_content(input, true)?;
        Ok(Content::only_rules(rules))
    }

    fn parse_path_block(input: ParseStream) -> Result<Self> {
        input.parse::<Token![in]>()?;
        let path: BlockPath = if input.peek(Token![*]) {
            input.parse::<Token![*]>()?;
            BlockPath::Clear
        } else {
            BlockPath::Path(input.parse()?)
        };

        let rules = Self::parse_block_content(input, true)?;
        Ok(Self::Content(Content { rules, path }))
    }

    fn parse_if_else(input: ParseStream) -> Result<Self> {
        input.parse::<Token![if]>()?;
        let cond = handle_expr(Expr::parse_without_eager_brace(input)?);
        let then_branch = Self::parse_block(input)?;

        let else_branch = if input.peek(Token![else]) {
            input.parse::<Token![else]>()?;
            if input.peek(Token![if]) {
                Self::parse_if_else(input)?
            } else {
                Self::Content(Self::parse_block(input)?)
            }
        } else {
            Self::Content(Content::only_rules(Vec::new()))
        };

        Ok(Self::If {
            cond,
            then: then_branch,
            else_branch: Box::new(else_branch),
        })
    }

    fn parse_match(input: ParseStream) -> Result<Self> {
        input.parse::<Token![match]>()?;
        let expr = handle_expr(Expr::parse_without_eager_brace(input)?);

        let content;
        syn::braced!(content in input);

        let mut arms = Vec::new();
        while !content.is_empty() {
            let pat = Pat::parse_multi_with_leading_vert(&content)?;
            content.parse::<Token![=>]>()?;

            let guard = if content.peek(Token![if]) {
                content.parse::<Token![if]>()?;
                Some(handle_expr(content.parse::<Expr>()?))
            } else {
                None
            };

            let body: Self = content.parse()?;
            arms.push((pat, guard, body));

            if !content.is_empty() {
                content.parse::<Token![,]>()?;
            }
        }

        Ok(Self::Match { expr, arms })
    }

    fn generate_style_content(&self, path: Option<&Path>) -> TokenStream {
        match self {
            Self::Rule { ty, args } => {
                let ty = if let Some(path) = path {
                    quote! { #path::#ty }
                } else {
                    quote! { #ty }
                };

                quote! {
                    __irisia_style_buffer.write(
                        &<#ty as ::irisia::style::StyleFn<_>>::from(
                            (#(#args,)*)
                        )
                    );
                }
            }

            Self::Let { name, expr } => {
                let closure_content = expr.wrap_closure(path);
                quote! {
                    let #name = #closure_content;
                }
            }

            Self::UseVar { value } => {
                quote! {
                    ::irisia::style::ReadStyle::read_style_into(&#value, __irisia_style_buffer);
                }
            }

            Self::Content(content) => content.generate_style_content(path),

            Self::If {
                cond,
                then,
                else_branch,
            } => {
                let then_content = then.generate_style_content(path);
                let else_content = else_branch.generate_style_content(path);
                quote! {
                    if #cond {
                        #then_content
                    } else {
                        #else_content
                    }
                }
            }

            Self::Match { expr, arms } => {
                let arm_tokens = arms.iter().map(|(pat, guard, body)| {
                    let body_content = body.generate_style_content(path);
                    let guard_content = guard.as_ref().map(|guard| {
                        quote! { if #guard { #guard } }
                    });

                    quote! {
                        #pat #guard_content => { #body_content }
                    }
                });

                quote! {
                    match #expr {
                        #(#arm_tokens)*
                    }
                }
            }
        }
    }

    fn wrap_closure(&self, path: Option<&Path>) -> TokenStream {
        let content = self.generate_style_content(path);
        quote! {
            move |__irisia_style_buffer: &mut ::irisia::style::StyleBuffer| {
                #content
            }
        }
    }
}

pub struct StyleMacro(StyleExpr);

impl Parse for StyleMacro {
    fn parse(input: ParseStream) -> Result<Self> {
        let content = StyleExpr::parse_block_content(input, false)?;
        Ok(StyleMacro(StyleExpr::Content(Content::only_rules(content))))
    }
}

impl StyleMacro {
    pub fn to_token_stream(&self) -> TokenStream {
        self.0.wrap_closure(None)
    }
}
