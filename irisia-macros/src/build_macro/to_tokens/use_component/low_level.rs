use crate::consts::*;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote, quote_spanned};
use syn::{Expr, Ident, Path, spanned::Spanned};

#[derive(Clone, Copy)]
pub enum Mode {
    Proxied,
    Direct,
}

pub enum LlExpr<'a> {
    UserDefined(&'a Expr),
    Complex(TokenStream),
}

pub struct LlField<'a> {
    pub name: &'a Ident,
    pub expr: LlExpr<'a>,
    pub mode: Mode,
}

pub struct LlGenerator<'a> {
    pub component_path: &'a Path,
    pub fields: Vec<LlField<'a>>,
    pub child_data: Option<&'a Expr>,
}

impl<'a> LlGenerator<'a> {
    pub fn generate(&self) -> TokenStream {
        let prop_assignments = self.fields.iter().map(assign_prop);

        let append_child_data = self.child_data.map(|child_data| {
            quote! {
                .set_child_data(#child_data)
            }
        });

        let empty = self.get_empty();
        let span = self.component_path.span();

        quote_spanned! {span=>
            {
                let __irisia_value = #empty;
                #(#prop_assignments)*

                #PATH_COMPONENT::UseComponent::new(__irisia_value)
                #append_child_data
            }
        }
    }

    fn get_empty(&self) -> TokenStream {
        let comp_path = self.component_path;
        quote! {
            {
                use #PATH_PROPERTY::Property as _;
                #PATH_PROPERTY::PropertyAgent::get_empty(
                    #comp_path::__irisia_prop_agent()
                )
            }
        }
    }
}

fn assign_prop(LlField { expr, mode, name }: &LlField) -> TokenStream {
    let span = expr.span();
    let path_private = PATH_PRIVATE.spanned(span);
    let coerce_hook = COERCE_HOOK.spanned(span);

    let definition = match mode {
        Mode::Proxied => {
            quote_spanned! {span=>
                #path_private::new_proxy_signal(#expr)
                    .get()
                    .coerce_unsize_helped(|x| {
                        __irisia_value.#name(x);
                    })(|x| #coerce_hook(x))
            }
        }
        Mode::Direct => {
            quote_spanned! {span=>
                #path_private::DirectAssign(#expr)
            }
        }
    };

    quote_spanned! {span=>
        let __irisia_value = __irisia_value
            .#name(#definition)
            .apply(__irisia_value);
    }
}

fn binary_fold<T, F>(slice: &[T], for_each: &F) -> TokenStream
where
    F: Fn(&T) -> TokenStream,
{
    match slice {
        [] => quote! {()},
        [one] => for_each(one),
        _ => {
            let (a, b) = slice.split_at(slice.len() / 2);
            let a = binary_fold(a, for_each);
            let b = binary_fold(b, for_each);
            quote! {(#a, #b)}
        }
    }
}

impl ToTokens for LlExpr<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::UserDefined(e) => e.to_tokens(tokens),
            Self::Complex(t) => t.to_tokens(tokens),
        }
    }
}
