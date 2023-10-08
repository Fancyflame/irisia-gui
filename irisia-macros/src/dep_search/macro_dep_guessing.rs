use proc_macro2::{Ident, TokenTree};
use syn::{
    parenthesized,
    parse::{Parse, ParseBuffer, ParseStream, Parser, Peek},
    punctuated::Punctuated,
    Expr, ItemUse, Signature, Token, Visibility,
};

use super::DepSearcher;

impl DepSearcher {
    pub(super) fn guess_macro_dep(&mut self, mut input: ParseBuffer) -> syn::Result<()> {
        #[must_use]
        fn try_skip<F, R, P>(input: &mut ParseBuffer, parser: F, peek: P) -> bool
        where
            F: FnOnce(ParseStream) -> syn::Result<R>,
            P: Peek,
        {
            if !input.peek(peek) {
                return false;
            }

            let forked = input.fork();
            if parser(&forked).is_ok() {
                *input = forked;
                true
            } else {
                false
            }
        }

        while let (Self::Some(watch_list), false) = (&mut *self, input.is_empty()) {
            if try_skip(&mut input, Visibility::parse, Token![pub])
                || try_skip(&mut input, ItemUse::parse, Token![use])
                || try_skip(&mut input, parse_impl_fn, Token![fn])
            {
                continue;
            }

            if input.peek(Token![self]) {
                if input.peek2(Token![::]) {
                    input.parse::<Token![self]>()?;
                    input.parse::<Token![::]>()?;
                    continue;
                }

                let forked = input.fork();
                if let Ok(field) = parse_self_call(&forked) {
                    watch_list.insert(field);
                    input = forked;
                    continue;
                } else {
                    *self = Self::WatchAll;
                    return Ok(());
                }
            }

            if let TokenTree::Group(group) = input.parse::<TokenTree>().unwrap() {
                let parser = |content: ParseStream| {
                    self.guess_macro_dep(content.fork()).unwrap();
                    Ok(())
                };
                let _ = parser.parse2(group.stream());
            }
        }

        Ok(())
    }
}

fn parse_self_call(input: ParseStream) -> syn::Result<Ident> {
    input.parse::<Token![self]>()?;
    input.parse::<Token![.]>()?;
    let field: Ident = input.parse()?;

    let content;
    parenthesized!(content in input);
    Punctuated::<Expr, Token![,]>::parse_terminated(&content)?;

    Ok(field)
}

fn parse_impl_fn(input: ParseStream) -> syn::Result<()> {
    input.parse::<Signature>()?;
    input
        .step(|cursor| {
            let c = match cursor.group(proc_macro2::Delimiter::Brace) {
                Some((_, _, next)) => next,
                None => *cursor,
            };

            Ok(((), c))
        })
        .unwrap();
    Ok(())
}

#[test]
fn test_custom_macro() {
    use std::collections::HashSet;
    use syn::parse_quote;

    let ds = DepSearcher::new(&parse_quote! {{
        macro_call! {
            chaotic syntax in macro => {
                pub(self) use my_module::{self, foo};

                foo(format!("at time {}", self.time()));
                () => pub(in self::some_mod) (self.function() + fn method(&self) {
                    1 + self.not_captured();
                    illegal syntax
                })

                123 => self.boolean();
            }
        }
    }});

    let set: HashSet<String> = match &ds {
        DepSearcher::Some(set) => set.iter().map(|id| id.to_string()).collect(),
        DepSearcher::WatchAll => unreachable!(),
    };

    assert!(set.contains("time"));
    assert!(set.contains("function"));
    assert!(set.contains("boolean"));
    assert!(!set.contains("not_captured"));
}
