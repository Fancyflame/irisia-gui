use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{token::Paren, TypePath};

use crate::expr::ConditionalApplicator;

pub struct DefaultConditionalApplicator {
    count: usize,
    skipped: usize,
    path: TypePath,
}

impl DefaultConditionalApplicator {
    pub fn new(count: usize, branch_path: TypePath) -> Self {
        DefaultConditionalApplicator {
            count,
            skipped: 0,
            path: branch_path,
        }
    }
}

impl ConditionalApplicator for DefaultConditionalApplicator {
    fn apply(&mut self, tokens: &mut TokenStream, other: impl ToTokens) {
        if self.skipped == self.count {
            panic!("inner error: branches requested more than count");
        }

        let path = &self.path;

        let stream = if self.skipped == self.count - 1 {
            other.to_token_stream()
        } else {
            quote!(#path::ArmA(#other))
        };

        add_arm2(tokens, path, self.skipped, &stream);
        self.skipped += 1;
    }
}

fn add_arm2(tokens: &mut TokenStream, path: &TypePath, depth: usize, expr: &TokenStream) {
    if depth == 0 {
        expr.to_tokens(tokens);
        return;
    }

    quote!(#path::ArmB).to_tokens(tokens);

    Paren::default().surround(tokens, |tokens| {
        add_arm2(tokens, path, depth - 1, expr);
    });
}
