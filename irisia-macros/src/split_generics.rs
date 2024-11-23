use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    punctuated::{Pair, Punctuated},
    ConstParam, GenericParam, Generics, Ident, Lifetime, LifetimeParam, Token, TypeParam,
    WhereClause,
};

type PunctuatedComma<T> = Punctuated<T, Token![,]>;

pub struct SplittedGenerics {
    pub lifetime_impl_generics: PunctuatedComma<LifetimeParam>,
    pub type_impl_generics: PunctuatedComma<TypeConstParam>,
    pub lifetime_type_generics: PunctuatedComma<Lifetime>,
    pub type_type_generics: PunctuatedComma<Ident>,
    pub where_clause: Option<WhereClause>,
    pub lt_token: Option<Token![<]>,
    pub gt_token: Option<Token![>]>,
}

#[derive(Clone)]
pub enum TypeConstParam {
    Type(TypeParam),
    Const(ConstParam),
}

impl ToTokens for TypeConstParam {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Const(c) => c.to_tokens(tokens),
            Self::Type(t) => t.to_tokens(tokens),
        }
    }
}

impl SplittedGenerics {
    pub fn split_from_generics(generics: Generics) -> Self {
        let mut out = SplittedGenerics {
            lifetime_impl_generics: Punctuated::new(),
            type_impl_generics: Punctuated::new(),
            lifetime_type_generics: Punctuated::new(),
            type_type_generics: Punctuated::new(),
            where_clause: generics.where_clause,
            lt_token: generics.lt_token,
            gt_token: generics.gt_token,
        };

        fn push<T>(p: &mut PunctuatedComma<T>, value: T, sep: Token![,]) {
            p.push_value(value);
            p.push_punct(sep);
        }

        for pair in generics.params.into_pairs() {
            let (value, sep) = match pair {
                Pair::Punctuated(v, t) => (v, t),
                Pair::End(v) => (v, Default::default()),
            };

            match value {
                GenericParam::Lifetime(l) => {
                    push(&mut out.lifetime_type_generics, l.lifetime.clone(), sep);
                    push(&mut out.lifetime_impl_generics, l, sep);
                }
                GenericParam::Const(mut c) => {
                    push(&mut out.type_type_generics, c.ident.clone(), sep);
                    c.default = None;
                    push(&mut out.type_impl_generics, TypeConstParam::Const(c), sep);
                }
                GenericParam::Type(mut t) => {
                    push(&mut out.type_type_generics, t.ident.clone(), sep);
                    t.default = None;
                    push(&mut out.type_impl_generics, TypeConstParam::Type(t), sep);
                }
            }
        }

        out
    }
}
