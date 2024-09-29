use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{Expr, Ident, Token};

use super::def_body::{DefBody, OptGroupSeg, Seg};

pub fn compile_from(
    tokens: &mut TokenStream,
    name: &Ident,
    variant: Option<&Ident>,
    body: &DefBody,
) {
    let mut gfo = GenFromOptions {
        tokens,
        name,
        variant,
        body,
    };
    let mut stack = vec![];
    let stream = &body.opt_groups;
    generate_trait_from(&mut gfo, &stack, stream);
    compile_from_recursive(&mut gfo, &mut stack, stream);
}

fn compile_from_recursive<'a>(
    opt: &mut GenFromOptions,
    parsed_opt: &mut Vec<(&'a Seg, Option<&'a Expr>)>,
    mut stream: &'a [OptGroupSeg],
) {
    while let Some((first, rest)) = stream.split_first() {
        stream = rest;
        match first {
            OptGroupSeg::Def { seg, .. } => {
                parsed_opt.push((seg, None));
                generate_trait_from(opt, parsed_opt, stream);
            }
            &OptGroupSeg::GroupStart { len: group_len } => {
                if group_len == 0 {
                    continue;
                }

                let path_len = parsed_opt.len();
                parsed_opt.extend(stream[..group_len].iter().map(|ogs| match ogs {
                    OptGroupSeg::Def { seg, default } => (seg, Some(default)),
                    OptGroupSeg::GroupStart { .. } => unreachable!(),
                }));
                compile_from_recursive(opt, parsed_opt, &stream[group_len..]);
                parsed_opt.truncate(path_len);
            }
        }
    }
}

struct GenFromOptions<'a> {
    tokens: &'a mut TokenStream,
    name: &'a Ident,
    variant: Option<&'a Ident>,
    body: &'a DefBody,
}

fn generate_trait_from(
    GenFromOptions {
        tokens,
        name,
        variant,
        body,
    }: &mut GenFromOptions,
    parsed: &Vec<(&Seg, Option<&Expr>)>,
    rest: &[OptGroupSeg],
) {
    let fields = {
        let nec = body.necessaries.iter().map(|seg| (seg, None));
        let orph = body
            .orphan_opts
            .iter()
            .map(|(seg, default)| (seg, Some(default)));
        let rest = rest.iter().filter_map(|ogs| match ogs {
            OptGroupSeg::Def { seg, default } => Some((seg, Some(default))),
            OptGroupSeg::GroupStart { .. } => None,
        });
        nec.chain(parsed.iter().copied()).chain(rest).chain(orph)
    };

    let req_fields = fields
        .clone()
        .filter_map(|(seg, default)| default.is_none().then_some(seg));

    let input_names = req_fields.clone().map(|seg| &seg.name);
    let input_types = {
        let types = req_fields.map(|seg| &seg.ty);
        quote! {
            (#(#types,)*)
        }
    };

    let field_inits = fields.map(|(Seg { name, ty: _ }, default)| {
        let colon = default.is_some().then(<Token![:]>::default);
        quote! {
            #name #colon #default,
        }
    });

    let span = match variant {
        Some(v) => v.span(),
        None => name.span(),
    };

    let colon2 = variant.is_some().then(<Token![::]>::default);
    quote_spanned! {
        span =>
        impl ::std::convert::From<#input_types> for #name {
            fn from((#(#input_names,)*): #input_types) -> Self {
                Self #colon2 #variant {
                    #(#field_inits)*
                }
            }
        }
    }
    .to_tokens(tokens);
}
