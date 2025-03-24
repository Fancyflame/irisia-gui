use super::{
    BlockStmt, Component, DomMacro, FieldAssignment, FieldDefinition, FieldType, ForStmt, IfStmt,
    MatchArm, MatchStmt, Stmt, UseSlot, WhileStmt,
};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    token::Brace,
    Error, Expr, Generics, Ident, Pat, Result, Token, Type,
};

mod kw {
    syn::custom_keyword!(model);
}

impl Parse for DomMacro {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        let generics: Generics = input.parse()?;

        let content;
        braced!(content in input);

        let mut fields: Vec<FieldDefinition> = Vec::new();
        while let Some(prefix) = parse_field_prefix(&content)? {
            fields.push(parse_field_def(&content, prefix)?);
            content.parse::<Token![,]>()?;
        }

        let body = parse_stmts(&content)?;

        Ok(DomMacro {
            name,
            generics,
            fields,
            body,
        })
    }
}

fn parse_field_def(input: ParseStream, prefix: FieldIdentPrefix) -> Result<FieldDefinition> {
    match prefix {
        FieldIdentPrefix::Value(name) => {
            input.parse::<Token![:]>()?;

            let from_ty: Type = input.parse()?;
            let to_ty = if input.peek(Token![=>]) {
                input.parse::<Token![=>]>()?;
                input.parse()?
            } else {
                from_ty.clone()
            };

            if let (Type::Infer(_), Type::Infer(_)) = (&from_ty, &to_ty) {
                return Err(Error::new(
                    input.span(),
                    "at least one type of from-position or to-position must be given",
                ));
            }

            Ok(FieldDefinition {
                name,
                field_type: FieldType::Value { from_ty, to_ty },
            })
        }
        FieldIdentPrefix::Model(name) => Ok(FieldDefinition {
            name,
            field_type: FieldType::Model,
        }),
    }
}

fn parse_stmts(input: ParseStream) -> Result<Vec<Stmt>> {
    let mut stmts = Vec::new();
    while !input.is_empty() {
        let stmt = parse_stmt(input)?;
        stmts.push(stmt);
    }
    Ok(stmts)
}

fn parse_stmt(input: ParseStream) -> Result<Stmt> {
    if input.peek(Token![if]) {
        Ok(Stmt::If(parse_if_stmt(input)?))
    } else if input.peek(Token![match]) {
        Ok(Stmt::Match(parse_match_stmt(input)?))
    } else if input.peek(Token![for]) {
        Ok(Stmt::For(parse_for_stmt(input)?))
    } else if input.peek(Token![while]) {
        Ok(Stmt::While(parse_while_stmt(input)?))
    } else if input.peek(Ident) && input.peek2(Token![;]) {
        Ok(Stmt::Slot(parse_slot(input)?))
    } else if input.peek(Brace) {
        Ok(Stmt::Block(parse_block(input)?))
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
                parse_block(&content)?
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

    Ok(ForStmt {
        pattern,
        expr,
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

fn parse_slot(input: ParseStream) -> Result<UseSlot> {
    let var = input.parse()?;
    input.parse::<Token![;]>()?;
    Ok(UseSlot { var })
}

fn parse_component(input: ParseStream) -> Result<Component> {
    let path: syn::Path = input.parse()?;
    let content;
    braced!(content in input);

    let mut fields = Vec::new();

    while let Some(prefix) = parse_field_prefix(&content)? {
        content.parse::<Token![:]>()?;

        let fa = match prefix {
            FieldIdentPrefix::Value(name) => FieldAssignment::Value {
                name,
                value: content.parse()?,
            },
            FieldIdentPrefix::Model(name) => FieldAssignment::Model {
                name,
                tree: parse_stmt(&content)?,
            },
        };
        content.parse::<Token![,]>()?;
        fields.push(fa);
    }

    Ok(Component {
        path,
        fields,
        body: parse_stmts(&content)?,
    })
}

enum FieldIdentPrefix {
    Value(Ident),
    Model(Ident),
}

fn parse_field_prefix(input: ParseStream) -> Result<Option<FieldIdentPrefix>> {
    if input.peek(kw::model) && input.peek2(Ident) {
        input.parse::<kw::model>()?;
        let name = input.parse()?;
        Ok(Some(FieldIdentPrefix::Model(name)))
    } else if input.peek(Ident) && input.peek2(Token![:]) {
        let name = input.parse()?;
        Ok(Some(FieldIdentPrefix::Value(name)))
    } else {
        Ok(None)
    }
}

fn parse_block(input: ParseStream) -> Result<BlockStmt> {
    let content;
    braced!(content in input);

    Ok(BlockStmt {
        stmts: parse_stmts(&content)?,
    })
}
