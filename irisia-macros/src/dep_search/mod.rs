use std::collections::HashSet;

use syn::{
    parse::ParseStream,
    visit::{self, Visit},
    Expr, ExprPath, Ident, ItemFn, Stmt,
};

#[cfg(feature = "macro-dep-guessing")]
mod macro_dep_guessing;

#[cfg(not(feature = "macro-dep-guessing"))]
mod min_macro_dep_guessing;

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
            self.guess_macro_dep(input.fork()).unwrap();
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
}

#[test]
fn test_stmts() {
    use syn::parse_quote;

    let ds = DepSearcher::new(&parse_quote! {{
        pub(self) use self::deeper::Foo;

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
