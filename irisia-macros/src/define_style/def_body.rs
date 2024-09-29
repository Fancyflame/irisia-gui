use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token::Bracket,
    Error, Expr, Ident, Result, Token, Type,
};

pub struct DefBody {
    pub necessaries: Vec<Seg>,
    pub opt_groups: Vec<OptGroupSeg>,
    pub orphan_opts: Vec<(Seg, Expr)>,
}

#[derive(Clone, Debug)]
pub struct Seg {
    pub name: Ident,
    pub ty: Type,
}

#[derive(Debug)]
pub enum OptGroupSeg {
    GroupStart { len: usize },
    Def { seg: Seg, default: Expr },
}

enum State {
    Required,
    Optional,
    OrphanOpt,
}

impl Parse for DefBody {
    fn parse(input: ParseStream) -> Result<Self> {
        Parser::new(input).parse()
    }
}

impl DefBody {
    pub fn optional_args(&self) -> impl Iterator<Item = (&Seg, &Expr)> + Clone {
        self.opt_groups
            .iter()
            .filter_map(|group_seg| {
                if let OptGroupSeg::Def { seg, default } = group_seg {
                    Some((seg, default))
                } else {
                    None
                }
            })
            .chain(self.orphan_opts.iter().map(|(x, y)| (x, y)))
    }
}

struct Parser<'a> {
    body: DefBody,
    state: State,
    input: ParseStream<'a>,
}

impl<'a> Parser<'a> {
    fn new(input: ParseStream<'a>) -> Self {
        Self {
            body: DefBody {
                necessaries: Vec::new(),
                opt_groups: Vec::new(),
                orphan_opts: Vec::new(),
            },
            state: State::Required,
            input,
        }
    }

    fn parse(mut self) -> Result<DefBody> {
        let input = self.input;
        while !input.peek(Token![;]) {
            if input.peek(Token![/]) {
                self.parse_pos_stop()?;
            } else if input.peek(Bracket) {
                self.parse_opt_group()?;
            } else {
                let (seg, default) = parse_seg(self.input)?;
                match default {
                    None => self.push_pos_arg(seg)?,
                    Some(default) => self.push_opt_arg(seg, default),
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            } else {
                break;
            }
        }
        input.parse::<Token![;]>()?;

        Ok(self.body)
    }

    fn parse_opt_group(&mut self) -> Result<()> {
        let content;
        let bracket = bracketed!(content in self.input);

        let header_index = match self.state {
            State::Required | State::Optional => {
                self.state = State::Optional;
                let index = self.body.opt_groups.len();
                self.body
                    .opt_groups
                    .push(OptGroupSeg::GroupStart { len: 0 });
                index
            }
            State::OrphanOpt => {
                return Err(Error::new(
                    bracket.span.span(),
                    "optional group are disallowed after position-arg-stop token `/`",
                ));
            }
        };

        while !content.is_empty() {
            let (seg, Some(default)) = parse_seg(&content)? else {
                return Err(content.error(
                    "expected `=` here, arguments inside optional group must have default value",
                ));
            };

            self.body.opt_groups.push(OptGroupSeg::Def { seg, default });

            if !content.is_empty() {
                content.parse::<Token![,]>()?;
            }
        }

        self.body.opt_groups[header_index] = OptGroupSeg::GroupStart {
            len: self.body.opt_groups.len() - (header_index + 1),
        };

        Ok(())
    }

    fn push_opt_arg(&mut self, seg: Seg, default: Expr) {
        match self.state {
            State::Required | State::Optional => {
                self.body.opt_groups.push(OptGroupSeg::Def { seg, default });
                self.state = State::Optional;
            }
            State::OrphanOpt => {
                self.body.orphan_opts.push((seg, default));
            }
        }
    }

    fn push_pos_arg(&mut self, seg: Seg) -> Result<()> {
        if let State::Required = self.state {
            self.body.necessaries.push(seg);
            Ok(())
        } else {
            Err(Error::new_spanned(
                &seg.name,
                "necessary arguments cannot be declared after optional arguments",
            ))
        }
    }

    fn parse_pos_stop(&mut self) -> Result<()> {
        let stop_token = self.input.parse::<Token![/]>()?;
        if let State::OrphanOpt = self.state {
            return Err(Error::new_spanned(
                stop_token,
                "duplicated position-argument-stop token `/` found",
            ));
        }

        self.state = State::OrphanOpt;
        Ok(())
    }
}

fn parse_seg(input: ParseStream) -> Result<(Seg, Option<Expr>)> {
    let seg = Seg {
        name: input.parse()?,
        ty: input.parse()?,
    };

    let default: Option<Expr> = if input.peek(Token![=]) {
        input.parse::<Token![=]>()?;
        Some(input.parse()?)
    } else {
        None
    };

    Ok((seg, default))
}
