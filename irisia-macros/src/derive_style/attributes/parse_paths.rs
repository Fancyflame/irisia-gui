use syn::{
    bracketed, parenthesized,
    parse::{ParseStream, Parser},
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Bracket, Paren},
    ExprPath, Ident, LitInt, LitStr, Member, Result, Token, Type,
};

#[derive(Clone, Debug)]
pub enum Segment {
    Member(Member),
    Fn {
        bind: Member,
        path: ExprPath,
        arg_types: Punctuated<Type, Token![,]>,
    },
}

impl Segment {
    pub const fn tag(&self) -> &Member {
        match self {
            Self::Member(m) => m,
            Self::Fn { bind, .. } => bind,
        }
    }
}

pub fn parse_paths(input: ParseStream) -> Result<Vec<Vec<Segment>>> {
    let lit_str: LitStr = input.parse()?;

    let parser = |input: ParseStream| parse_style_path(input, &[PathRecorder::new()]);
    parser
        .parse_str(&lit_str.value())
        .map(|recorders| recorders.into_iter().map(|x| x.path).collect())
}

#[derive(Clone)]
struct PathRecorder {
    path: Vec<Segment>,
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

    fn push_seg(&mut self, seg: Segment) -> Result<()> {
        if self.next_is_comma {
            Err(syn::Error::new(
                match &seg {
                    Segment::Member(m) => m.span(),
                    Segment::Fn { bind, .. } => bind.span(),
                },
                "expect a comma",
            ))
        } else {
            self.path.push(seg);
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

                let seg = if input.peek(Token![:]) {
                    input.parse::<Token![:]>()?;
                    Segment::Fn {
                        bind: member,
                        path: input.parse()?,
                        arg_types: {
                            let content;
                            parenthesized!(content in input);
                            Punctuated::parse_terminated(&content)?
                        },
                    }
                } else {
                    Segment::Member(member)
                };

                for r in &mut recorders {
                    r.push_seg(seg.clone())?;
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
