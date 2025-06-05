use crate::component::FieldAssignMethod;

use super::{
    BlockStmt, BuildMacro, ComponentStmt, FieldAssignment, ForStmt, IfStmt, MatchArm, MatchStmt,
    Stmt, UseExprStmt, WhileStmt,
};

use proc_macro2::TokenStream;
use syn::{
    Error, Expr, Ident, Pat, Result, Token, braced, parenthesized,
    parse::{Parse, ParseStream},
    token::{Brace, Paren},
};

mod kw {
    use syn::custom_keyword;

    custom_keyword!(key);
}

impl Parse for BuildMacro {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            stmts: parse_stmts(input)?,
        })
    }
}

fn parse_stmts(input: ParseStream) -> Result<Vec<Stmt>> {
    let mut stmts = Vec::new();
    while !input.is_empty() {
        stmts.push(parse_stmt(input, true)?);
    }
    Ok(stmts)
}

fn parse_stmt(input: ParseStream, multiple_mode: bool) -> Result<Stmt> {
    if input.peek(Token![if]) {
        Ok(Stmt::If(parse_if_stmt(input)?))
    } else if input.peek(Token![match]) {
        Ok(Stmt::Match(parse_match_stmt(input)?))
    } else if input.peek(Token![for]) {
        Ok(Stmt::For(parse_for_stmt(input)?))
    } else if input.peek(Token![while]) {
        Ok(Stmt::While(parse_while_stmt(input)?))
    } else if input.peek(Brace) {
        Ok(Stmt::Block(parse_block(input)?))
    } else if input.peek(Paren) {
        Ok(Stmt::UseExpr(parse_use_expr(input)?))
    } else {
        Ok(Stmt::Component(parse_component(input)?))
    }
}

fn parse_if_stmt(input: ParseStream) -> Result<IfStmt> {
    input.parse::<Token![if]>()?;
    let condition = syn::Expr::parse_without_eager_brace(input)?;
    let body = parse_block(input)?;

    let else_body = if input.peek(Token![else]) {
        input.parse::<Token![else]>()?;

        let lookahead = input.lookahead1();
        let stmt = if lookahead.peek(Token![if]) {
            Stmt::If(parse_if_stmt(input)?)
        } else if lookahead.peek(Brace) {
            Stmt::Block(parse_block(input)?)
        } else {
            return Err(lookahead.error());
        };

        Some(Box::new(stmt))
    } else {
        None
    };

    Ok(IfStmt {
        condition,
        then_branch: body,
        else_branch: else_body,
    })
}

fn parse_match_stmt(input: ParseStream) -> Result<MatchStmt> {
    input.parse::<Token![match]>()?;
    let expr = input.call(Expr::parse_without_eager_brace)?;
    let content;
    braced!(content in input);

    let mut arms = Vec::new();
    while !content.is_empty() {
        arms.push(MatchArm {
            pattern: content.call(Pat::parse_multi_with_leading_vert)?,
            guard: if content.peek(Token![if]) {
                content.parse::<Token![if]>()?;
                Some(content.parse()?)
            } else {
                None
            },
            body: {
                content.parse::<Token![=>]>()?;
                parse_stmt(&content, false)?
            },
        });
        content.parse::<Token![,]>()?;
    }
    Ok(MatchStmt { expr, arms })
}

fn parse_for_stmt(input: ParseStream) -> Result<ForStmt> {
    input.parse::<Token![for]>()?;
    let pattern = input.call(Pat::parse_multi_with_leading_vert)?;
    input.parse::<Token![in]>()?;
    let expr = input.call(Expr::parse_without_eager_brace)?;
    let get_key = if input.peek(Token![,]) {
        input.parse::<Token![,]>()?;
        input.parse::<kw::key>()?;
        input.parse::<Token![=]>()?;
        Some(input.call(Expr::parse_without_eager_brace)?)
    } else {
        None
    };

    Ok(ForStmt {
        pattern,
        expr,
        get_key,
        body: parse_block(input)?,
    })
}

fn parse_while_stmt(input: ParseStream) -> Result<WhileStmt> {
    input.parse::<Token![while]>()?;
    Ok(WhileStmt {
        condition: input.call(Expr::parse_without_eager_brace)?,
        body: parse_block(&input)?,
    })
}

enum FieldAssignmentName {
    Super(Token![super]),
    Ident(Ident),
}

fn parse_field_assignment(
    input: ParseStream,
) -> Result<Option<FieldAssignment<FieldAssignmentName>>> {
    if !((input.peek(Ident) || input.peek(Token![super]))
        && input.peek2(Token![:])
        && !input.peek3(Token![:]))
    {
        return Ok(None);
    };

    let name = if input.peek(Token![super]) {
        FieldAssignmentName::Super(input.parse()?)
    } else {
        FieldAssignmentName::Ident(input.parse()?)
    };

    input.parse::<Token![:]>()?;

    let method = if input.peek(Token![=]) {
        input.parse::<Token![=]>()?;
        FieldAssignMethod::Direct
    } else {
        FieldAssignMethod::HostingSignal
    };

    let value = input.parse()?;
    input.parse::<Token![,]>()?;

    Ok(Some(FieldAssignment {
        name,
        value,
        method,
    }))
}

fn parse_component(input: ParseStream) -> Result<ComponentStmt> {
    let comp_type = input.parse()?;

    let content;
    braced!(content in input);

    let mut fields = Vec::new();
    let mut child_data = None;

    while let Some(fa) = parse_field_assignment(&content)? {
        match fa.name {
            FieldAssignmentName::Ident(ident) => fields.push(FieldAssignment {
                name: ident,
                value: fa.value,
                method: fa.method,
            }),
            FieldAssignmentName::Super(super_token) => match child_data {
                Some(_) => {
                    return Err(Error::new_spanned(
                        super_token,
                        "cannot define child data duplicatedly",
                    ));
                }
                None => match fa.method {
                    FieldAssignMethod::HostingSignal => child_data = Some(fa.value),
                    FieldAssignMethod::Direct => {
                        return Err(Error::new_spanned(
                            super_token,
                            "cannot use `:=` on child data",
                        ));
                    }
                },
            },
        }
    }

    Ok(ComponentStmt {
        comp_type,
        child_data,
        fields,
        body: parse_stmts(&content)?,
    })
}

fn parse_block(input: ParseStream) -> Result<BlockStmt> {
    let content;
    braced!(content in input);

    Ok(BlockStmt {
        stmts: parse_stmts(&content)?,
    })
}

fn parse_use_expr(input: ParseStream) -> Result<UseExprStmt> {
    let paren_content;
    parenthesized!(paren_content in input);
    let expr_is_empty = paren_content.is_empty();
    let value: Option<TokenStream> = if expr_is_empty {
        None
    } else {
        Some(paren_content.parse()?)
    };

    Ok(UseExprStmt { value })
}
