use std::collections::{hash_map::Entry, HashMap};

use pat_bind::PatBinds;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Expr, Ident};

mod el_dec;
mod parse;
mod pat_bind;
mod to_tokens;

pub struct Environment {
    vars: Vec<Ident>,
    accessable: HashMap<Ident, usize>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            vars: Vec::new(),
            accessable: HashMap::new(),
        }
    }

    fn pop_env(&mut self, count: usize) {
        let discard_vars = self.vars.drain(self.vars.len() - count..);
        for v in discard_vars {
            match self.accessable.entry(v) {
                Entry::Occupied(mut occ) => {
                    let rest = occ.get_mut();
                    if *rest > 1 {
                        *rest -= 1;
                    } else {
                        occ.remove();
                    }
                }
                _ => unreachable!("environment variable not present"),
            }
        }
    }

    fn push_env(&mut self, env: Ident) {
        self.vars.push(env.clone());
        self.accessable
            .entry(env)
            .and_modify(|x| *x += 1)
            .or_insert(1);
    }

    fn bind_env<F, R>(&mut self, pat: &PatBinds, f: F) -> R
    where
        F: FnOnce(&mut Environment) -> R,
    {
        for ident in &pat.binds {
            self.push_env(ident.clone());
        }
        let ret = f(self);
        self.pop_env(pat.binds.len());
        ret
    }

    fn clone_env_wires(&self) -> TokenStream {
        clone_env_raw(self.accessable.keys())
    }

    fn borrow_wire(&self, expr: &Expr) -> TokenStream {
        let vars = self.accessable.keys();
        let vars2 = vars.clone();
        quote! {{
            #[allow(unused_variables)]
            let (#(#vars,)*) = (#(&#vars2,)*);
            #expr
        }}
    }

    fn deref_wire_in_user_expr(&self) -> TokenStream {
        let (pat, value) = self.deref_wire_in_user_expr_splitted();
        quote! {
            #[allow(unused_variables)]
            let #pat = #value;
        }
    }

    fn deref_wire_in_user_expr_splitted(&self) -> (TokenStream, TokenStream) {
        let idents = self.accessable.keys();
        let idents2 = idents.clone();
        (
            quote! {
                (#(#idents,)*)
            },
            quote! {
                (#(
                    ::irisia::data_flow::watch_on_deref::WatchOnDeref::new(&#idents2),
                )*)
            },
        )
    }

    fn create_wire(&self, expr: &Expr) -> TokenStream {
        let env = self.clone_env_wires();
        let deref_env = self.deref_wire_in_user_expr();
        quote_spanned! { expr.span() =>
            {
                #env
                ::irisia::data_flow::wire(move || {
                    #deref_env
                    let __irisia_wire_ret = {#expr};
                    __irisia_wire_ret
                })
            }
        }
    }
}

fn clone_env_raw<'a, I>(vars: I) -> TokenStream
where
    I: Iterator<Item = &'a Ident> + Clone,
{
    let vars2 = vars.clone();
    quote! {
        #[allow(unused_variables)]
        let (#(#vars,)*) = {
            use ::irisia::__macro_helper::CloneHelper as _;
            (#(#vars2.__irisia_clone_wire(),)*)
        };
    }
}
