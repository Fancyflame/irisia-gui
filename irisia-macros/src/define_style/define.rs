use std::collections::{hash_map::Entry, HashMap};

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::ParseStream, Attribute, Error, Ident, Result, Token, Type};

use super::{
    def_body::{DefBody, Seg},
    impl_from::compile_from,
};

pub fn parse(input: ParseStream) -> Result<TokenStream> {
    let mut map: HashMap<Ident, Definition> = HashMap::new();
    while !input.is_empty() {
        let header = Definition::parse_header(input)?;
        let body: DefBody = input.parse()?;

        match map.entry(header.name.clone()) {
            Entry::Occupied(mut occ) => {
                occ.get_mut().append(header, body)?;
            }
            Entry::Vacant(vac) => {
                vac.insert(Definition::new(header, body));
            }
        }
    }

    let mut tokens = TokenStream::new();
    for def in map.values() {
        def.compile().to_tokens(&mut tokens);
        def.compile_methods()?.to_tokens(&mut tokens);
    }
    Ok(tokens)
}

struct Definition {
    attrs: Vec<Attribute>,
    name: Ident,
    variants: Variants,
}

enum Variants {
    StructLike(DefBody),
    EnumLike(HashMap<Ident, DefBody>),
}

struct DefHeader {
    attrs: Vec<Attribute>,
    name: Ident,
    variant: Option<Ident>,
}

impl Definition {
    fn new(header: DefHeader, body: DefBody) -> Self {
        Self {
            attrs: header.attrs,
            name: header.name,
            variants: match header.variant {
                Some(id) => {
                    let mut map = HashMap::new();
                    map.insert(id, body);
                    Variants::EnumLike(map)
                }
                None => Variants::StructLike(body),
            },
        }
    }

    fn parse_header(input: ParseStream) -> Result<DefHeader> {
        let attrs = Attribute::parse_outer(input)?;

        let name: Ident = input.parse()?;
        let variant: Option<Ident> = if input.peek(Token![::]) {
            input.parse::<Token![::]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        input.parse::<Token![:]>()?;
        Ok(DefHeader {
            attrs,
            name,
            variant,
        })
    }

    fn append(&mut self, header: DefHeader, body: DefBody) -> Result<()> {
        assert_eq!(self.name, header.name);

        let (map, new_var) = match (&mut self.variants, header.variant) {
            (Variants::EnumLike(map), Some(new_var)) => (map, new_var),
            (Variants::StructLike(_), _) => {
                let name = &self.name;
                return Err(Error::new_spanned(
                    name,
                    format!(
                        "style `{name}` has already declared struct-liked. \
                        if you want this style can be overloaded, \
                        you should provide a variant name like `{name}::VariantName` \
                        instead of `{name}` to let it become enum-liked."
                    ),
                ));
            }
            (Variants::EnumLike(_), None) => {
                let name = &header.name;
                return Err(Error::new_spanned(
                    name,
                    format!(
                        "style `{name}` indeed was declared enum-liked before, \
                        but the overload style did not. \
                        if you want to overload this style, \
                        you should provide a variant name like `{name}::VariantName` \
                        instead of `{name}` to let it become enum-liked as well."
                    ),
                ));
            }
        };

        match map.entry(new_var) {
            Entry::Occupied(occ) => {
                return Err(Error::new_spanned(
                    occ.key(),
                    format!("cannot re-define `{}::{}`", header.name, occ.key()),
                ))
            }
            Entry::Vacant(vac) => {
                vac.insert(body);
            }
        }

        self.attrs.extend(header.attrs);

        Ok(())
    }

    fn compile(&self) -> TokenStream {
        let Self {
            attrs,
            name,
            variants,
        } = self;

        match variants {
            Variants::EnumLike(e) => {
                let field_names = e.keys();
                let field_bodies = e.values().map(|x| Self::compile_fields(x, false));

                let mut tokens = quote! {
                    #(#attrs)*
                    pub enum #name {
                        #(#field_names #field_bodies,)*
                    }
                };

                for (variant, body) in e {
                    compile_from(&mut tokens, name, Some(variant), body);
                }

                tokens
            }

            Variants::StructLike(body) => {
                let fields = Self::compile_fields(body, true);

                let mut tokens = quote! {
                    #(#attrs)*
                    pub struct #name #fields
                };

                compile_from(&mut tokens, name, None, body);
                tokens
            }
        }
    }

    fn compile_fields(body: &DefBody, pub_token: bool) -> TokenStream {
        let all_fields = body
            .necessaries
            .iter()
            .chain(body.optional_args().map(|(seg, _)| seg));

        let field_names = all_fields.clone().map(|seg| &seg.name);
        let types = all_fields.map(|seg| &seg.ty);
        let pub_token = if pub_token {
            Some(<Token![pub]>::default())
        } else {
            None
        };

        quote! {
            { #(#pub_token #field_names: #types,)* }
        }
    }

    fn compile_methods(&self) -> Result<TokenStream> {
        let mut methods: Vec<(&Ident, &Type, TokenStream)> = Vec::new();
        match &self.variants {
            Variants::StructLike(s) => {
                for (Seg { name, ty }, _) in s.optional_args() {
                    methods.push((
                        name,
                        ty,
                        quote! {
                            self.#name = value;
                        },
                    ));
                }
            }

            Variants::EnumLike(enums) => {
                let mut opt_map: HashMap<&Ident, (&Type, bool)> = HashMap::new();

                let mut first_variant = None;
                for (variant, body) in enums.iter() {
                    for (
                        Seg {
                            name: arg_name,
                            ty: arg_type,
                        },
                        _,
                    ) in body.optional_args()
                    {
                        let Some(first_variant) = first_variant else {
                            opt_map.insert(arg_name, (arg_type, false));
                            continue;
                        };

                        match opt_map.get_mut(arg_name) {
                            Some((expect_type, checked)) => {
                                if *arg_type != **expect_type {
                                    let et = expect_type.to_token_stream().to_string();
                                    let at = arg_type.to_token_stream().to_string();
                                    return Err(Error::new_spanned(arg_name, format!(
                                        "type of the optional argument `{arg_name}` defined in variant `{variant}` \
                                        is conflicts with other variants.\n\
                                        expected: {et}\n\
                                           found: {at}"
                                    )));
                                }
                                *checked = true;
                            }
                            None => {
                                return Err(not_all_defined_error(arg_name, first_variant));
                            }
                        }
                    }

                    if first_variant.is_none() {
                        first_variant = Some(variant);
                        continue;
                    }

                    for (&arg_name, (_, checked)) in opt_map.iter_mut() {
                        if !*checked {
                            return Err(not_all_defined_error(arg_name, variant));
                        }
                        *checked = false;
                    }
                }

                for (&method, &(ty, _)) in opt_map.iter() {
                    let variants = enums.keys();
                    methods.push((
                        method,
                        ty,
                        quote! {
                            let (#(| Self::#variants { #method: ref_mut, .. })*) = self;
                            *ref_mut = value;
                        },
                    ));
                }
            }
        }

        let name = &self.name;
        let methods = methods.iter().map(|(method, ty, content)| {
            quote! {
                pub fn #method(&mut self, value: #ty) -> &mut Self {
                    #content
                    self
                }
            }
        });

        Ok(quote! {
            impl #name {
                #(#methods)*
            }
        })
    }
}

fn not_all_defined_error(arg_name: &Ident, example: &Ident) -> Error {
    Error::new_spanned(
        arg_name,
        format!(
            "optional argument `{arg_name}` is not defined by other variants: `{example}`.\n\
            note: all variants must contain the same optional arguments, \
            except the position that can be different."
        ),
    )
}
