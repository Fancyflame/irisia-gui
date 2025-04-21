use crate::component::FieldAssignMethod;

use super::{
    BlockStmt, BuildMacro, ComponentStmt, FieldAssignment, ForStmt, IfStmt, MatchArm, MatchStmt,
    Stmt, UseExprStmt, WhileStmt, check_has_parent_props_assigned,
};

use proc_macro2::{Span, TokenStream};
use syn::{
    Error, Expr, Ident, Pat, Path, Result, Token, braced, bracketed, parenthesized,
    parse::{Parse, ParseStream},
    token::{Brace, Bracket, Paren},
};

mod kw {
    use syn::custom_keyword;

    custom_keyword!(key);
}

impl Parse for BuildMacro {
    fn parse(input: ParseStream) -> Result<Self> {
        let virtual_parent = parse_virtual_parent(input)?;
        let stmts = parse_stmts(input)?;
        if virtual_parent.is_none() && stmts.iter().any(check_has_parent_props_assigned) {
            return Err(Error::new(
                Span::call_site(),
                "parent-property declaration is not allowed at root \
                when there is no parent type provided, which is default to be `()`. \
                consider using `in path::to::Type;` to declare a virtual parent.",
            ));
        }

        Ok(Self {
            stmts,
            virtual_parent,
        })
    }
}

fn parse_virtual_parent(input: ParseStream) -> Result<Option<Path>> {
    if !input.peek(Token![in]) {
        return Ok(None);
    }

    input.parse::<Token![in]>()?;
    let path: Path = input.parse()?;
    input.parse::<Token![;]>()?;
    Ok(Some(path))
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

fn parse_field_assignment(input: ParseStream) -> Result<Option<FieldAssignment>> {
    let mut method = if input.peek(Bracket) {
        FieldAssignMethod::ParentProp
    } else if input.peek(Ident) {
        FieldAssignMethod::HostingSignal
    } else {
        return Ok(None);
    };

    if !input.peek2(Token![:]) {
        return Ok(None);
    }

    let name: Ident = if let FieldAssignMethod::ParentProp = method {
        let content;
        bracketed!(content in input);
        content.parse()?
    } else {
        input.parse()?
    };
    input.parse::<Token![:]>()?;

    if input.peek(Token![=]) {
        let eq_token = input.parse::<Token![=]>()?;
        if let FieldAssignMethod::ParentProp = method {
            return Err(Error::new_spanned(
                eq_token,
                "direct-assign mode is not available when declaring parent prop",
            ));
        }
        method = FieldAssignMethod::Direct;
    }

    let value = input.parse()?;
    input.parse::<Token![,]>()?;

    Ok(Some(FieldAssignment {
        name,
        value,
        method,
    }))
}

fn parse_component(input: ParseStream) -> Result<ComponentStmt> {
    let type_path = input.parse()?;

    let content;
    braced!(content in input);

    let mut fields = Vec::new();
    while let Some(fa) = parse_field_assignment(&content)? {
        fields.push(fa);
    }

    Ok(ComponentStmt {
        type_path,
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
