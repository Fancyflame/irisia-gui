use std::collections::HashSet;

use crate::component::{ast::*, parse::parse_stmts};
use syn::{
    Error, Ident, Result, Token, braced, bracketed, parse::ParseStream, spanned::Spanned,
    token::Bracket,
};

mod kw {
    use syn::custom_keyword;

    custom_keyword!(event);
}

enum FieldAssignmentName {
    Super(Token![super]),
    Ident(Ident),
}

pub fn parse_component(input: ParseStream) -> Result<ComponentStmt> {
    let comp_type = input.parse()?;

    let content;
    braced!(content in input);

    let mut fields = Vec::new();
    let mut field_pool = HashSet::new();
    let mut child_data = None;

    while let Some(fa) = parse_field_assignment(&content)? {
        match fa.name {
            FieldAssignmentName::Ident(ident) => {
                if !field_pool.insert(ident.to_string()) {
                    return Err(Error::new(
                        ident.span(),
                        format!("field `{ident}` already exists"),
                    ));
                }

                fields.push(FieldAssignment {
                    name: ident,
                    value: fa.value,
                    decoration: fa.decoration,
                })
            }
            FieldAssignmentName::Super(super_token) => {
                if child_data.is_some() {
                    return Err(Error::new_spanned(
                        super_token,
                        "cannot define `super` property duplicatedly",
                    ));
                }
                if !matches!(fa.decoration, FieldDecoration::None) {
                    return Err(Error::new_spanned(
                        super_token,
                        "cannot use decoration on `super` property",
                    ));
                }
                child_data = Some(fa.value);
            }
        }
    }

    let body_span = content.span();
    let body = parse_stmts(&content)?;

    if !body.is_empty() && field_pool.contains("children") {
        return Err(Error::new(
            body_span,
            "cannot define child elements because `children` property has been defined manually",
        ));
    }

    Ok(ComponentStmt {
        comp_type,
        child_data,
        fields,
        body,
    })
}

fn parse_decoration(input: ParseStream) -> Result<FieldDecoration> {
    let content;
    let bracketed_token = bracketed!(content in input);

    let decoration = if content.is_empty() {
        FieldDecoration::None
    } else if content.peek(kw::event) {
        content.parse::<kw::event>()?;
        FieldDecoration::Event
    } else if content.peek(Token![=]) {
        content.parse::<Token![=]>()?;
        FieldDecoration::DirectAssign
    } else {
        return Err(Error::new(
            bracketed_token.span.span(),
            "unknown decoration",
        ));
    };

    Ok(decoration)
}

#[rustfmt::skip]
fn peek_prop(input: ParseStream) -> bool {
    (input.peek(Ident) || 
    input.peek(Token![super])) &&
    (
        (input.peek2(Token![:]) && !input.peek2(Token![::])) ||
        input.peek2(Bracket)
    )
}

fn parse_field_assignment(
    input: ParseStream,
) -> Result<Option<FieldAssignment<FieldAssignmentName>>> {
    if !peek_prop(input) {
        return Ok(None);
    };

    let name = if input.peek(Token![super]) {
        FieldAssignmentName::Super(input.parse()?)
    } else {
        FieldAssignmentName::Ident(input.parse()?)
    };

    let decoration = if input.peek(Bracket) {
        parse_decoration(input)?
    } else {
        FieldDecoration::None
    };

    input.parse::<Token![:]>()?;

    let value = input.parse()?;
    input.parse::<Token![,]>()?;

    Ok(Some(FieldAssignment {
        name,
        value,
        decoration,
    }))
}
