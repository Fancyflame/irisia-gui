use std::collections::{hash_map::Entry, HashMap};

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens, TokenStreamExt};
use syn::{parse::ParseStream, Error, Expr, Ident, Result, Token, Type};

pub fn parse(input: ParseStream) -> Result<TokenStream> {
    let mut map: HashMap<Ident, Definition> = HashMap::new();
    while !input.is_empty() {
        let header = Definition::parse_header(input)?;
        let body = Definition::parse_body(input)?;

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
    name: Ident,
    variants: Variants,
}

#[derive(Clone)]
struct Seg {
    name: Ident,
    ty: Type,
}

enum Variants {
    StructLike(DefBody),
    EnumLike(HashMap<Ident, DefBody>),
}

struct DefHeader {
    name: Ident,
    variant: Option<Ident>,
}

struct DefBody {
    necessaries: Vec<Seg>,
    optionals: Vec<(Seg, Expr)>,
    opt_arg_max_len: usize,
}

enum State {
    Necessary,
    Optional,
    RequireKV,
}

impl Definition {
    fn new(header: DefHeader, body: DefBody) -> Self {
        Self {
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
        let name: Ident = input.parse()?;
        let variant: Option<Ident> = if input.peek(Token![::]) {
            input.parse::<Token![::]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        input.parse::<Token![:]>()?;
        Ok(DefHeader { name, variant })
    }

    fn parse_body(input: ParseStream) -> Result<DefBody> {
        let mut body = DefBody {
            necessaries: Vec::new(),
            optionals: Vec::new(),
            opt_arg_max_len: 0,
        };

        let mut state = State::Necessary;
        while !input.is_empty() {
            if input.peek(Token![/]) {
                let stop_token = input.parse::<Token![/]>()?;
                match state {
                    State::RequireKV => {
                        return Err(Error::new_spanned(
                            stop_token,
                            "duplicated position-argument-stop token `/` found",
                        ));
                    }
                    _ => {
                        body.opt_arg_max_len = body.optionals.len();
                    }
                }
                state = State::RequireKV;
            } else {
                let seg = Seg {
                    name: input.parse()?,
                    ty: input.parse()?,
                };

                match state {
                    State::Necessary if !input.peek(Token![=]) => {
                        body.necessaries.push(seg);
                    }
                    _ => {
                        if let State::Necessary = state {
                            state = State::Optional;
                        }
                        input.parse::<Token![=]>()?;
                        body.optionals.push((seg, input.parse()?));
                    }
                }
            }

            if input.peek(Token![;]) {
                input.parse::<Token![;]>()?;
                break;
            } else {
                input.parse::<Token![,]>()?;
            }
        }

        if !matches!(state, State::RequireKV) {
            body.opt_arg_max_len = body.optionals.len();
        }

        Ok(body)
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

        Ok(())
    }

    fn compile(&self) -> TokenStream {
        let name = &self.name;
        match &self.variants {
            Variants::EnumLike(e) => {
                let field_names = e.keys();
                let field_bodies = e.values().map(|x| Self::compile_fields(x, false));
                let impl_from = e
                    .iter()
                    .map(|(variant, body)| Self::compile_from(name, Some(variant), body));

                quote! {
                    pub enum #name {
                        #(#field_names #field_bodies,)*
                    }

                    #(#impl_from)*
                }
            }

            Variants::StructLike(body) => {
                let fields = Self::compile_fields(body, true);
                let impl_from = Self::compile_from(name, None, body);
                quote! {
                    pub struct #name #fields
                    #impl_from
                }
            }
        }
    }

    fn compile_fields(body: &DefBody, pub_token: bool) -> TokenStream {
        let all_fields = body
            .necessaries
            .iter()
            .chain(body.optionals.iter().map(|(seg, _)| seg));
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

    fn compile_from(name: &Ident, variant: Option<&Ident>, body: &DefBody) -> TokenStream {
        let mut tokens = TokenStream::new();
        let pos_opts = &body.optionals[..body.opt_arg_max_len];

        for got_opts_count in 0..=body.opt_arg_max_len {
            let got_opts = body
                .necessaries
                .iter()
                .chain(pos_opts[..got_opts_count].iter().map(|(seg, _)| seg));

            let names = got_opts.clone().map(|seg| &seg.name);
            let names2 = names.clone();
            let types = got_opts.map(|seg| &seg.ty);
            let types2 = types.clone();

            let rest_opts =
                body.optionals[got_opts_count..]
                    .iter()
                    .map(|(Seg { name, .. }, default)| {
                        quote! {#name: #default}
                    });

            let span = match variant {
                Some(v) => v.span(),
                None => name.span(),
            };

            let colon2 = variant.is_some().then(<Token![::]>::default);

            tokens.append_all(quote_spanned! {
                span =>
                impl ::std::convert::From<(#(#types,)*)> for #name {
                    fn from((#(#names,)*): (#(#types2,)*)) -> Self {
                        Self #colon2 #variant {
                            #(#names2,)*
                            #(#rest_opts,)*
                        }
                    }
                }
            });
        }

        tokens
    }

    fn compile_methods(&self) -> Result<TokenStream> {
        let mut methods: Vec<(&Ident, &Type, TokenStream)> = Vec::new();
        match &self.variants {
            Variants::StructLike(s) => {
                for (Seg { name, ty }, _) in &s.optionals {
                    methods.push((
                        name,
                        ty,
                        quote! {
                            self.#name = value;
                        },
                    ));
                }
            }

            Variants::EnumLike(e) => {
                let mut map: HashMap<&Ident, (&Type, Vec<&Ident>)> = HashMap::new();

                for (variant, body) in e.iter() {
                    for (
                        Seg {
                            name: arg_name,
                            ty: arg_type,
                        },
                        _,
                    ) in &body.optionals
                    {
                        match map.entry(arg_name) {
                            Entry::Occupied(mut occ) => {
                                let (expect_type, defined_variants) = occ.get_mut();
                                if *arg_type != **expect_type {
                                    let display_vars = display_variants(defined_variants);
                                    return Err(Error::new_spanned(arg_name, format!(
                                        "type of the optional argument `{arg_name}` defined in variant `{variant}` \
                                        is conflicts with other variants: {display_vars}"
                                    )));
                                }
                                defined_variants.push(variant);
                            }
                            Entry::Vacant(vac) => {
                                vac.insert((arg_type, vec![variant]));
                            }
                        }
                    }
                }

                for (&method, &(ty, ref variants)) in map.iter() {
                    let display_vars = format!(
                        "style `{}` only with these variants could call `{method}`: {}",
                        self.name,
                        display_variants(variants)
                    );

                    methods.push((
                        method,
                        ty,
                        quote! {
                            match self {
                                #(| Self::#variants { #method: ref_mut, .. })* => *ref_mut = value,
                                _ => panic!(#display_vars),
                            }
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

fn display_variants(vec: &Vec<&Ident>) -> String {
    vec.iter()
        .map(|id| format!("`{id}`"))
        .collect::<Vec<_>>()
        .join(", ")
}
