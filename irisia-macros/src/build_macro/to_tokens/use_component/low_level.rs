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
    Generated(TokenStream),
}

pub enum LlFields<'a> {
    AllFromValue(&'a Expr),
    Detailed(Vec<LlField<'a>>),
}

pub struct LlField<'a> {
    pub name: &'a Ident,
    pub expr: LlExpr<'a>,
    pub mode: Mode,
}

pub struct LlGenerator<'a> {
    pub component_path: &'a Path,
    pub fields: LlFields<'a>,
    pub child_data: Option<&'a Expr>,
}

impl<'a> LlGenerator<'a> {
    pub fn generate(&self) -> TokenStream {
        let span = self.component_path.span();

        let get_value = match &self.fields {
            LlFields::AllFromValue(v) => quote_spanned! {span=>
                let __irisia_value = #PATH_PRIVATE::DirectAssign(#v);
            },
            LlFields::Detailed(d) => self.value_from_detailed_fields(d),
        };

        let append_child_data = self.child_data.map(|child_data| {
            quote! {
                .set_child_data(#child_data)
            }
        });

        quote_spanned! {span=>
            {
                #get_value
                #PATH_COMPONENT::UseComponent::new(__irisia_value)
                #append_child_data
            }
        }
    }

    fn value_from_detailed_fields(&self, detailed: &Vec<LlField>) -> TokenStream {
        let comp_path = self.component_path;
        let prop_assignments = detailed.iter().map(assign_prop);
        let span = comp_path.span();

        quote_spanned! {span=>
            let __irisia_value = {
                use #PATH_PROPERTY::Property as _;
                #comp_path::__IRISIA_EMPTY_PROP
            };
            #(#prop_assignments)*
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

/* fn binary_fold<T, F>(slice: &[T], for_each: &F) -> TokenStream
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
} */

impl ToTokens for LlExpr<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::UserDefined(e) => e.to_tokens(tokens),
            Self::Generated(t) => t.to_tokens(tokens),
        }
    }
}
