use std::{collections::HashMap, fmt::Display};

use attr_parser_fn::{
    find_attr,
    meta::{conflicts, key_value, path_only, ParseMetaExt},
    ParseArgs, ParseAttrTrait,
};
use proc_macro2::{Delimiter, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    spanned::Spanned, Attribute, Error, Expr, Field, Fields, Generics, Ident, Index, LitStr,
    Member, Result,
};

use super::style_path;

pub enum FieldInit {
    AlwaysRequires,
    Optional,
    OptionalWith(Expr),
}

struct FieldInfo<'a> {
    origin: &'a Field,
    init: FieldInit,
}

struct PathDef {
    from_tuple_order: Vec<Member>,
    defined_len: usize,
}

pub struct StyleDefinition<'a> {
    all_fields: HashMap<Member, FieldInfo<'a>>,
    init_delimeter: Delimiter,
    paths: Vec<PathDef>,
}

impl<'a> StyleDefinition<'a> {
    pub fn parse_fields(top_attrs: &[Attribute], fields: &'a Fields) -> Result<Self> {
        let mut this = Self {
            all_fields: HashMap::with_capacity(fields.len()),
            paths: Vec::new(),
            init_delimeter: match fields {
                Fields::Named(_) => Delimiter::Brace,
                Fields::Unnamed(_) => Delimiter::Parenthesis,
                Fields::Unit => Delimiter::None,
            },
        };

        for (i, field) in fields.iter().enumerate() {
            this.extract_field_init(field, i as u32)?;
        }

        this.extract_paths(top_attrs)?;
        Ok(this)
    }

    fn extract_field_init(&mut self, field: &'a Field, nth: u32) -> Result<()> {
        let Some(attr) = find_attr::only(&field.attrs, "style")? else {
            return Ok(());
        };

        let default = ParseArgs::new()
            .meta(
                conflicts((
                    ("default", path_only()).map(|_| FieldInit::Optional),
                    ("default", key_value::<Expr>()).map(FieldInit::OptionalWith),
                ))
                .optional(),
            )
            .parse_attrs(attr)?
            .meta
            .unwrap_or(FieldInit::AlwaysRequires);

        let member = match &field.ident {
            Some(ident) => Member::Named(ident.clone()),
            None => Member::Unnamed(Index {
                index: nth,
                span: field.span(),
            }),
        };

        self.all_fields.insert(
            member,
            FieldInfo {
                origin: field,
                init: default,
            },
        );

        Ok(())
    }

    fn extract_paths(&mut self, attrs: &[Attribute]) -> Result<()> {
        let path_exprs = ParseArgs::new()
            .rest_args::<Vec<LitStr>>()
            .parse_concat_attrs(find_attr::all(attrs, "style"))?
            .rest_args;

        let mut unused_fields = HashMap::new();

        for path_expr in path_exprs {
            let raw_paths = path_expr.parse_with(style_path::parse)?;

            for path in raw_paths {
                debug_assert!(unused_fields.is_empty());
                unused_fields.extend(self.all_fields.iter());

                for member in &path {
                    if unused_fields.remove(member).is_none() {
                        return Err(Error::new_spanned(
                            member,
                            format!(
                                "field `{}` {}",
                                display_member(member),
                                if self.all_fields.contains_key(member) {
                                    "already used"
                                } else {
                                    "not found"
                                }
                            ),
                        ));
                    }
                }

                let mut from_tuple_order = path;
                let defined_len = from_tuple_order.len();

                for (unused, fi) in unused_fields.drain() {
                    if let FieldInit::AlwaysRequires = &fi.init {
                        return Err(Error::new_spanned(
                            unused,
                            format!(
                                "field `{}` should always be initialized, \
                                but is not initialized in at least one style path. \
                                if it is explicitly not required, \
                                use `#[style(default = \"...\")]` to specify a default value",
                                display_member(unused)
                            ),
                        ));
                    }
                    from_tuple_order.push(unused.clone());
                }

                self.paths.push(PathDef {
                    from_tuple_order,
                    defined_len,
                });
            }
        }

        Ok(())
    }

    fn compile(
        &self,
        ident: &Ident,
        variant_name: Option<&Ident>,
        generics: &Generics,
    ) -> TokenStream {
        let mut tokens = TokenStream::new();
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let init_path = if let Some(variant_name) = variant_name {
            quote! { Self::#variant_name }
        } else {
            quote! { Self }
        };
        let mut sort_buffer: Vec<Option<TokenStream>> = Vec::new();

        for path_def @ &PathDef {
            ref from_tuple_order,
            defined_len,
        } in &self.paths
        {
            let tuple_type = from_tuple_order
                .iter()
                .take(defined_len)
                .map(|member| &self.all_fields[member].origin.ty);
            let tuple_type = quote! { (#(#tuple_type,)*) };

            let body = match self.init_delimeter {
                Delimiter::Brace => self.compile_named_body(path_def),
                Delimiter::Parenthesis => self.compile_unnamed_body(&mut sort_buffer, path_def),
                Delimiter::None => quote! {},
                Delimiter::Bracket => unreachable!(),
            };

            quote! {
                impl #impl_generics ::std::convert::From<tuple_type>
                    for #ident #ty_generics
                #where_clause
                {
                    fn from(__irisia_from: #tuple_type) -> Self {
                        #init_path #body
                    }
                }
            }
            .to_tokens(&mut tokens);
        }

        tokens
    }

    fn compile_named_body(
        &self,
        &PathDef {
            ref from_tuple_order,
            defined_len,
        }: &PathDef,
    ) -> TokenStream {
        let defined_fields = (0..defined_len).map(|index| {
            let index = Index::from(index);
            quote! { __irisia_from.#index }
        });

        let undefined_fields = from_tuple_order
            .iter()
            .skip(defined_len)
            .map(|member| self.all_fields[member].init.compile());

        let field_idents = from_tuple_order.iter();
        let field_values = defined_fields.chain(undefined_fields);
        quote! {
            {
                #(#field_idents: #field_values,)*
            }
        }
    }

    fn compile_unnamed_body(
        &self,
        sort_buffer: &mut Vec<Option<TokenStream>>,
        &PathDef {
            ref from_tuple_order,
            defined_len,
        }: &PathDef,
    ) -> TokenStream {
        debug_assert!(sort_buffer.is_empty());
        sort_buffer.resize(self.all_fields.len(), None);

        let mut input_tuple = from_tuple_order.iter().map(|member| match member {
            Member::Unnamed(index) => (index.index as usize, member),
            Member::Named(_) => unreachable!(),
        });

        // cannot swap iterators as the former will attempt to advance first
        for (input_index, (target_index, _)) in (0..defined_len).zip(&mut input_tuple) {
            sort_buffer[target_index] = Some(quote! {
                __irisia_from.#input_index
            });
        }

        for (target_index, uninit_member) in input_tuple {
            let default_behavior = self.all_fields[uninit_member].init.compile();
            sort_buffer[target_index] = Some(default_behavior);
        }

        let args = sort_buffer.drain(..).map(Option::unwrap);
        quote! {
            (#(#args,)*)
        }
    }
}

fn display_member(member: &Member) -> &dyn Display {
    match member {
        Member::Named(name) => name,
        Member::Unnamed(index) => &index.index,
    }
}

impl FieldInit {
    fn compile(&self) -> TokenStream {
        match self {
            Self::AlwaysRequires => unreachable!(),
            Self::Optional => quote! {
                ::std::default::Default::default()
            },
            Self::OptionalWith(expr) => expr.to_token_stream(),
        }
    }
}
