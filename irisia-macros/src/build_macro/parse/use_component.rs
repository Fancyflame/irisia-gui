use std::collections::HashSet;

use crate::build_macro::{ast::*, parse::parse_stmts};
use syn::{
    Error, Ident, Result, Token, braced, bracketed, parse::ParseStream,
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
                })
            }
            FieldAssignmentName::Super(super_token) => {
                if child_data.is_some() {
                    return Err(Error::new_spanned(
                        super_token,
                        "cannot define `super` property duplicatedly",
                    ));
                }

                if let FieldValue::Proxied(expr) = fa.value {
                    child_data = Some(expr);
                } else {
                    return Err(Error::new_spanned(
                        super_token,
                        "cannot use decoration on `super` property",
                    ));
                }
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

fn parse_field_value(flag: Option<ParseStream>, input: ParseStream) -> Result<FieldValue> {
    let flag = match flag {
        Some(flag) if !flag.is_empty() => flag,
        _ => return input.parse().map(FieldValue::Proxied),
    };

    if flag.peek(kw::event) {
        flag.parse::<kw::event>()?;
        Ok(FieldValue::Event)
    } else if flag.peek(Token![=]) {
        flag.parse::<Token![=]>()?;
        input.parse().map(FieldValue::DirectAssign)
    } else if flag.peek(Token![..]) {
        flag.parse::<Token![..]>()?;
        Ok(FieldValue::UseNested)
    } else {
        Err(Error::new(
            flag.span(),
            "unknown decoration",
        ))
    }
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

    let flag = if input.peek(Bracket) {
        let content;
        bracketed!(content in input);
        Some(content)
    } else {
        None
    };

    input.parse::<Token![:]>()?;
    let value = parse_field_value(flag.as_ref(), input)?;
    input.parse::<Token![,]>()?;

    Ok(Some(FieldAssignment { name, value }))
}
