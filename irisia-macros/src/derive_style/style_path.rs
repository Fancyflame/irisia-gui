use syn::{bracketed, parse::ParseStream, token::Bracket, Member, Result};

pub fn parse(input: ParseStream) -> Result<Vec<Vec<Member>>> {
    let mut template_path = Vec::new();
    parse_style_path(input, &mut template_path)?;

    let mut out = Vec::new();
    flatten_path_recursive(&template_path, vec![], &mut out);
    Ok(out)
}

fn flatten_path_recursive(
    mut template_path: &[Seg],
    mut current_path: Vec<Member>,
    out: &mut Vec<Vec<Member>>,
) {
    loop {
        match template_path.split_first() {
            Some((first, rest)) => {
                template_path = rest;
                match first {
                    Seg::Field(field) => {
                        current_path.push(field.clone());
                    }
                    Seg::Optional { span } => {
                        flatten_path_recursive(&template_path[*span..], current_path.clone(), out);
                        flatten_path_recursive(template_path, current_path, out);
                        return;
                    }
                }
            }
            None => {
                out.push(current_path);
                return;
            }
        }
    }
}

#[derive(Clone)]
enum Seg {
    Field(Member),
    Optional { span: usize },
}

fn parse_style_path(input: ParseStream, out: &mut Vec<Seg>) -> Result<()> {
    while !input.is_empty() {
        if input.peek(Bracket) {
            let content;
            bracketed!(content in input);
            out.push(Seg::Optional { span: 0 });
            let old_len = out.len();
            parse_style_path(&content, out)?;
            out[old_len - 1] = Seg::Optional {
                span: out.len() - old_len,
            };
            continue;
        }

        let member = input.parse()?;
        out.push(Seg::Field(member));
    }

    Ok(())
}
