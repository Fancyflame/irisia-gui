use std::collections::HashSet;

use proc_macro2::{Ident, TokenTree};
use syn::{
    parenthesized,
    parse::{Parse, ParseBuffer, ParseStream, Parser, Peek},
    parse_quote,
    punctuated::Punctuated,
    visit::{self, Visit},
    Block, Expr, ExprPath, ImplItemFn, ItemFn, ItemUse, Token, Visibility,
};

pub enum DepSearcher {
    WatchAll,
    Some(HashSet<Ident>),
}

impl Visit<'_> for DepSearcher {
    fn visit_impl_item_fn(&mut self, _: &'_ syn::ImplItemFn) {}
    fn visit_item_fn(&mut self, _: &'_ ItemFn) {}

    fn visit_expr_method_call(&mut self, call: &syn::ExprMethodCall) {
        if self.is_watch_all() {
            return;
        }

        match &*call.receiver {
            Expr::Path(path) if is_self(path) => {
                if let Self::Some(vec) = self {
                    vec.insert(call.method.clone());
                }
            }
            _ => visit::visit_expr_method_call(self, call),
        }
    }

    fn visit_expr_path(&mut self, path: &syn::ExprPath) {
        if self.is_watch_all() {
            return;
        }

        if is_self(path) {
            *self = Self::WatchAll;
        } else {
            visit::visit_expr_path(self, path);
        }
    }

    fn visit_macro(&mut self, mac: &syn::Macro) {
        let _ = mac.parse_body_with(|input: ParseStream| {
            self.guess_macro_dep(input.fork()).unwrap();
            Ok(())
        });
    }
}

fn is_self(path: &ExprPath) -> bool {
    path.qself.is_none() && path.path.is_ident("self")
}

impl DepSearcher {
    pub fn new(block: &Block) -> Self {
        let mut this = Self::Some(HashSet::new());
        visit::visit_block(&mut this, block);
        this
    }

    fn is_watch_all(&self) -> bool {
        matches!(self, Self::WatchAll)
    }

    fn guess_macro_dep(&mut self, mut input: ParseBuffer) -> syn::Result<()> {
        #[must_use]
        fn try_skip<T: Parse, P: Peek>(input: &mut ParseBuffer, peek: P) -> bool {
            if !input.peek(peek) {
                return false;
            }

            let forked = input.fork();
            if forked.parse::<T>().is_ok() {
                *input = forked;
                true
            } else {
                false
            }
        }

        while let (Self::Some(watch_list), false) = (&mut *self, input.is_empty()) {
            if try_skip::<Visibility, _>(&mut input, Token![pub])
                || try_skip::<ItemUse, _>(&mut input, Token![use])
                || try_skip::<ImplItemFn, _>(&mut input, Token![fn])
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

#[test]
fn test_watch() {
    let block: Block = parse_quote! {{
        use self::deeper::macro_call;

        if self.show_text() == "" {
            println!("{} said nothing", self.name());
            return;
        }

        println!("{} said `{}`", self.name(), self.show_text());

        impl Foo {
            fn do_not_capture(&self){
                self.not_captured();
            }
        }

        macro_call! {
            custom syntax in macro => {
                pub(self) use self::some_mod::foo;
                foo(format!("at time {}", self.time()));
                () => pub(in self::some_mod) fn method(&self) {
                    1 + self.not_captured();
                }
            }
        }
    }};

    let ds = DepSearcher::new(&block);

    let DepSearcher::Some(set) = &ds
    else {
        unreachable!();
    };

    let set: HashSet<String> = set.iter().map(|id| id.to_string()).collect();

    assert!(set.contains("name"));
    assert!(set.contains("show_text"));
    assert!(set.contains("time"));
    assert_eq!(set.len(), 3);
}
