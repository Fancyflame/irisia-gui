use syn::{
    bracketed, parenthesized,
    parse::{ParseStream, Parser},
    spanned::Spanned,
    token::{Bracket, Paren},
    Error, Ident, Lit, LitInt, Member, Meta, MetaNameValue, Result, Token,
};

pub fn parse_paths(meta: Meta) -> Result<Option<Vec<Vec<Member>>>> {
    let lit_str = match meta {
        Meta::NameValue(MetaNameValue {
            lit: Lit::Str(lit_str),
            ..
        }) => lit_str,
        Meta::Path(_) => return Ok(None),
        _ => return Err(Error::new_spanned(meta, "unexpected list")),
    };

    let parser = |input: ParseStream| parse_style_path(input, &[PathRecorder::new()]);
    match parser.parse_str(&lit_str.value()) {
        Ok(recorders) => Ok(Some(recorders.into_iter().map(|x| x.path).collect())),
        Err(e) => Err(Error::new_spanned(lit_str, e)),
    }
}

#[derive(Clone)]
struct PathRecorder {
    path: Vec<Member>,
    next_is_comma: bool,
}

impl PathRecorder {
    fn new() -> Self {
        PathRecorder {
            path: Vec::new(),
            next_is_comma: false,
        }
    }

    fn assume_comma(&mut self, comma: &Token![,]) -> Result<()> {
        if self.next_is_comma {
            self.next_is_comma = false;
            Ok(())
        } else {
            Err(syn::Error::new(
                comma.span,
                "expect a member(integer for tuple, identity for struct)",
            ))
        }
    }

    fn push_field(&mut self, field: Member) -> Result<()> {
        if self.next_is_comma {
            Err(syn::Error::new(field.span(), "expect a comma"))
        } else {
            self.path.push(field);
            self.next_is_comma = true;
            Ok(())
        }
    }
}

fn parse_style_path(input: ParseStream, recorders: &[PathRecorder]) -> Result<Vec<PathRecorder>> {
    let mut output = Vec::with_capacity(recorders.len());

    loop {
        let mut recorders = recorders.to_owned();
        loop {
            if input.peek(Token![,]) {
                let comma = input.parse()?;
                for r in &mut recorders {
                    r.assume_comma(&comma)?;
                }
            } else if input.peek(Ident) || input.peek(LitInt) {
                let member: Member = input.parse()?;
                for r in &mut recorders {
                    r.push_field(member.clone())?;
                }
            } else if input.peek(Bracket) {
                let content;
                bracketed!(content in input);
                recorders.append(&mut parse_style_path(&content, &recorders)?);
            } else if input.peek(Paren) {
                let content;
                parenthesized!(content in input);
                recorders = parse_style_path(&content, &recorders)?;
            } else {
                break;
            }
        }

        output.append(&mut recorders);

        if !input.is_empty() {
            input.parse::<Token![|]>()?;
        } else {
            break;
        }
    }

    Ok(output)
}
