use std::collections::HashSet;

use syn::{
    parse::ParseStream,
    parse_quote,
    punctuated::Punctuated,
    visit::{self, Visit},
    Expr, ExprLit, ExprPath, Ident, ItemFn, Lit, Stmt, Token,
};

#[cfg(feature = "macro-dep-guessing")]
mod macro_dep_guessing;

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
        if mac.path.is_ident("macro_rules") {
            return;
        }

        let _ = mac.parse_body_with(|input: ParseStream| {
            self.guess_macro_dep(input).unwrap();
            Ok(())
        });
    }
}

fn is_self(path: &ExprPath) -> bool {
    path.qself.is_none() && path.path.is_ident("self")
}

impl DepSearcher {
    pub fn new(stmt: &Stmt) -> Self {
        let mut this = Self::Some(HashSet::new());
        visit::visit_stmt(&mut this, stmt);
        this
    }

    fn is_watch_all(&self) -> bool {
        matches!(self, Self::WatchAll)
    }

    #[cfg(not(feature = "macro-dep-guessing"))]
    fn guess_macro_dep(&mut self, input: ParseStream) -> syn::Result<()> {
        if !self.try_parse_format_syntax(input) {
            *self = Self::WatchAll;
        }
        Ok(())
    }

    #[must_use]
    fn try_parse_format_syntax(&mut self, input: ParseStream) -> bool {
        let mut args = match Punctuated::<Expr, Token![,]>::parse_terminated(&input) {
            Ok(p) => p.into_iter(),
            Err(_) => return false,
        };

        let formatter = match args.next() {
            Some(Expr::Lit(ExprLit {
                lit: Lit::Str(litstr),
                attrs: _,
            })) => litstr.value(),
            _ => return false,
        };

        let mut fmt_slice = &*formatter;
        while let Some(start) = fmt_slice.find("{") {
            fmt_slice = &fmt_slice[start + 1..];
            if fmt_slice.starts_with("{") {
                fmt_slice = &fmt_slice[1..];
                continue;
            }

            if fmt_slice.starts_with("self") {
                *self = Self::WatchAll;
                return true;
            }
        }

        for expr in args {
            visit::visit_expr(self, &expr);
        }

        true
    }
}

#[test]
fn test_stmts() {
    use syn::parse_quote;

    let ds = DepSearcher::new(&parse_quote! {{
        pub(self) use self::deeper::Foo;

        format!("do not capture this: {{self}}");

        if self.banned() {
            println!("`{}` is banned to post", self.name());
            return;
        }

        println!(
            "{} said {}",
            self.name(),
            format!("`{}`", self.show_text())
        );

        impl Foo {
            fn do_not_capture(&self) {
                self.not_captured();
            }
        }
    }});

    let set: HashSet<String> = match &ds {
        DepSearcher::Some(set) => set.iter().map(|id| id.to_string()).collect(),
        DepSearcher::WatchAll => unreachable!(),
    };

    assert!(set.contains("banned"));
    assert!(set.contains("name"));
    assert!(set.contains("show_text"));
    assert!(!set.contains("not_captured"));
}

#[test]
fn test_cap_self() {
    let ds = DepSearcher::new(&parse_quote! {
        println!("print self: `{self:?}`", self.some_field());
    });
    assert!(ds.is_watch_all());
}
