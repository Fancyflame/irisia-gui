use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt};
use syn::{GenericParam, Generics, WhereClause};

pub fn split_for_impl_unbracketed(
    g: &Generics,
) -> (
    ImplGenericsUnbracketed,
    TypeGenericsUnbracketed,
    Option<&WhereClause>,
) {
    (
        ImplGenericsUnbracketed(g),
        TypeGenericsUnbracketed(g),
        g.where_clause.as_ref(),
    )
}

fn tokens_or_default<T>(tokens: &mut TokenStream, append: Option<&T>)
where
    T: ToTokens + Default,
{
    match append {
        Some(t) => t.to_tokens(tokens),
        None => T::default().to_tokens(tokens),
    }
}

pub struct ImplGenericsUnbracketed<'a>(&'a Generics);

impl ToTokens for ImplGenericsUnbracketed<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if self.0.params.is_empty() {
            return;
        }

        // Print lifetimes before types and consts, regardless of their
        // order in self.params.
        for param in self.0.params.pairs() {
            if let GenericParam::Lifetime(_) = **param.value() {
                param.to_tokens(tokens);

                // Ensure the trailing punct
                tokens_or_default(tokens, param.punct().copied());
            }
        }

        for param in self.0.params.pairs() {
            if let GenericParam::Lifetime(_) = **param.value() {
                continue;
            }

            match param.value() {
                GenericParam::Lifetime(_) => unreachable!(),
                GenericParam::Type(param) => {
                    // Leave off the type parameter defaults

                    // tokens.append_all(param.attrs.outer()); // why `outer` here?
                    tokens.append_all(&param.attrs);

                    param.ident.to_tokens(tokens);
                    if !param.bounds.is_empty() {
                        tokens_or_default(tokens, param.colon_token.as_ref());
                        param.bounds.to_tokens(tokens);
                    }
                }
                GenericParam::Const(param) => {
                    // Leave off the const parameter defaults
                    tokens.append_all(&param.attrs);
                    param.const_token.to_tokens(tokens);
                    param.ident.to_tokens(tokens);
                    param.colon_token.to_tokens(tokens);
                    param.ty.to_tokens(tokens);
                }
            }

            // Ensure the trailing punct
            tokens_or_default(tokens, param.punct().copied());
        }
    }
}

pub struct TypeGenericsUnbracketed<'a>(&'a Generics);

impl<'a> ToTokens for TypeGenericsUnbracketed<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.0.params.is_empty() {
            return;
        }

        // Print lifetimes before types and consts, regardless of their
        // order in self.params.
        for param in self.0.params.pairs() {
            if let GenericParam::Lifetime(def) = *param.value() {
                // Leave off the lifetime bounds and attributes
                def.lifetime.to_tokens(tokens);

                // Ensure the trailing punct
                tokens_or_default(tokens, param.punct().copied());
            }
        }

        for param in self.0.params.pairs() {
            if let GenericParam::Lifetime(_) = **param.value() {
                continue;
            }

            match param.value() {
                GenericParam::Lifetime(_) => unreachable!(),
                GenericParam::Type(param) => {
                    // Leave off the type parameter defaults
                    param.ident.to_tokens(tokens);
                }
                GenericParam::Const(param) => {
                    // Leave off the const parameter defaults
                    param.ident.to_tokens(tokens);
                }
            }

            // Ensure the trailing punct
            tokens_or_default(tokens, param.punct().copied());
        }
    }
}
