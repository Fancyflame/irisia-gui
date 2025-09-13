use crate::consts::*;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Expr, Ident, Path};

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
        let defs_tuple = binary_fold(&self.fields, &make_definition_tuple_item);
        let names_tuple = binary_fold(&self.fields, &|f| f.name.to_token_stream());

        let append_child_data = self.child_data.map(|child_data| {
            quote! {
                .set_child_data(#child_data)
            }
        });

        let agent = self.get_agent();
        let template_prop = self.generate_template_prop();

        quote! {
            (
                #PATH_COMPONENT::UseComponent::new(
                    |#names_tuple| #PATH_PROPERTY::PropCast::<_>::prop_cast(
                        #agent,
                        #template_prop
                    ),
                    #defs_tuple,
                )
                #append_child_data
            )
        }
    }

    fn get_agent(&self) -> TokenStream {
        let comp_path = self.component_path;
        quote! {
            {
                use #PATH_PROPERTY::Property as _;
                #comp_path::__irisia_prop_agent()
            }
        }
    }

    pub fn generate_template_prop(&self) -> TokenStream {
        let agent = self.get_agent();
        let init_field = self.fields.iter().map(init_field);

        quote! {{
            let __irisia_agent = #agent;
            let __irisia_value = #PATH_PROPERTY::PropertyAgent::get_empty(__irisia_agent);
            #(#init_field)*
            __irisia_value
        }}
    }
}

fn make_definition_tuple_item(LlField { expr, mode, .. }: &LlField) -> TokenStream {
    match mode {
        Mode::Proxied => {
            quote! {
                #PATH_COMPONENT::definition::proxy_signal_helper::check_eq(#expr).get()
            }
        }
        Mode::Direct => {
            quote! {
                #PATH_COMPONENT::definition::DirectAssign(#expr)
            }
        }
    }
}

fn init_field(LlField { name, .. }: &LlField) -> TokenStream {
    quote! {
        let __irisia_value = #PATH_PROPERTY::PropUpdate::<_, _>::prop_update(
            __irisia_agent,
            __irisia_value,
            __irisia_agent.#name(#name),
        );
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
