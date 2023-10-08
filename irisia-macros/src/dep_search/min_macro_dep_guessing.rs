use syn::{parse::ParseBuffer, punctuated::Punctuated, visit, Expr, Token};

use super::DepSearcher;

impl DepSearcher {
    pub(super) fn guess_macro_dep(&mut self, input: ParseBuffer) -> syn::Result<()> {
        match Punctuated::<Expr, Token![,]>::parse_terminated(&input) {
            Ok(exprs) => {
                for expr in exprs {
                    visit::visit_expr(self, &expr);
                }
            }
            Err(_) => *self = Self::WatchAll,
        }

        Ok(())
    }
}
