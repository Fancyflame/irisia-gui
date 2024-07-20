use std::collections::HashMap;

use pat_bind::PatBinds;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, Ident, TypePath};

mod pat_bind;
mod kw {
    use syn::custom_keyword;

    custom_keyword!(input);
    custom_keyword!(key);
}
mod parse;
mod to_tokens;

pub struct Environment {
    vars: Vec<Ident>,
}

impl Environment {
    pub fn new() -> Self {
        Self { vars: Vec::new() }
    }

    fn pop_env(&mut self, count: usize) {
        self.vars.truncate(self.vars.len() - count);
    }

    fn bind_env<F, R>(&mut self, pat: &PatBinds, f: F) -> R
    where
        F: FnOnce(&mut Environment) -> R,
    {
        self.vars.extend(pat.binds.iter().cloned());
        let stack_size = pat.binds.len();
        let ret = f(self);
        self.pop_env(stack_size);
        ret
    }

    fn env_to_tokens(&self) -> TokenStream {
        let vars = &self.vars;
        quote! {
            #[allow(unused_variables)]
            let (#(#vars,)*) = (
                #(::std::clone::Clone::clone(&#vars),)*
            );
        }
    }

    fn create_wire(&self, expr: &Expr) -> TokenStream {
        let env = self.env_to_tokens();
        quote! {
            {
                #env
                ::irisia::data_flow::wire(move || {
                    #expr
                })
            }
        }
    }
}

struct ElementDeclaration {
    el_type: TypePath,
    wired_props: HashMap<Ident, Expr>,
    styles: Expr,
    on_create: Expr,
    slot: TokenStream,
}
