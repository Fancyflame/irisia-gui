use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    visit::{visit_pat, Visit},
    Expr, Ident, Pat,
};

pub struct PatBinds {
    pub pattern: Pat,
    pub guard: Option<Expr>,
    pub binds: Vec<Ident>,
    pub tuple_expr: TokenStream,
}

impl PatBinds {
    pub fn new(pattern: Pat, guard: Option<Expr>) -> Self {
        let mut binds = Vec::new();
        visit_pat(&mut Visitor(&mut binds), &pattern);

        let tuple_expr = quote! {
            (#(#binds,)*)
        };

        PatBinds {
            pattern,
            guard,
            binds,
            tuple_expr,
        }
    }

    pub fn bind_var_from_wire(&self, from_ident: &Ident, pattern: impl ToTokens) -> TokenStream {
        let Self { binds, .. } = self;

        quote! {
            #(
                #[allow(unused_variables)]
                let #binds = ::irisia::data_flow::ReadableExt::map(
                    ::std::clone::Clone::clone(&#from_ident),
                    |#[allow(unused_variables)] #pattern| {
                        #binds
                    }
                );
            )*
        }
    }
}

struct Visitor<'a>(&'a mut Vec<Ident>);

impl Visit<'_> for Visitor<'_> {
    fn visit_pat_ident(&mut self, i: &syn::PatIdent) {
        self.0.push(i.ident.clone());
        if let Some((_, sub)) = &i.subpat {
            self.visit_pat(&sub);
        }
    }

    // we do not need to implement `visit_pat_struct`, see `https://docs.rs/syn/latest/syn/struct.FieldPat.html#`
}
