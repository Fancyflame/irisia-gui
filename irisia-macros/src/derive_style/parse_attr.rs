use std::{collections::HashMap, fmt::Display};

use attr_parser_fn::{
    find_attr,
    meta::{conflicts, key_value, list, path_only, ParseMetaExt},
    ParseArgs, ParseAttrTrait,
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    Attribute, Error, Expr, Field, Fields, Generics, Ident, Index, LitStr, Member, Path, Result,
    Type,
};

use super::style_path;

pub enum FieldInit {
    AlwaysRequired,
    Optional,
    OptionalWith(Expr),
}

struct FieldInfo<'a> {
    origin: &'a Field,
    init: FieldInit,
    map: Option<(Type, Path)>,
}

struct PathDef {
    from_tuple_order: Vec<Member>,
    defined_len: usize,
}

pub struct StyleDefinition<'a> {
    all_fields: HashMap<Member, FieldInfo<'a>>,
    paths: Vec<PathDef>,
    empty_path: PathDef,
    derive_default: bool,
}

pub fn derive_for(
    top_attrs: &[Attribute],
    ident: &Ident,
    variant_name: Option<&Ident>,
    generics: &Generics,
    fields: &Fields,
) -> Result<TokenStream> {
    let def = StyleDefinition::parse_fields(top_attrs, fields)?;
    Ok(def.compile(ident, variant_name, generics))
}

impl<'a> StyleDefinition<'a> {
    fn parse_fields(top_attrs: &[Attribute], fields: &'a Fields) -> Result<Self> {
        let mut this = Self {
            all_fields: HashMap::with_capacity(fields.len()),
            paths: Vec::new(),
            empty_path: PathDef {
                from_tuple_order: Vec::new(),
                defined_len: 0,
            },
            derive_default: false,
        };

        for (member, field) in fields.members().zip(fields.iter()) {
            if let Some(info) = this
                .all_fields
                .insert(member.clone(), Self::extract_field_init(field)?)
            {
                return Err(Error::new_spanned(info.origin, "duplicate field"));
            }

            this.empty_path.from_tuple_order.push(member.clone());
        }

        this.load_paths(top_attrs)?;
        Ok(this)
    }

    fn extract_field_init(field: &'a Field) -> Result<FieldInfo> {
        let (default, map_args) = if let Some(attr) = find_attr::only(&field.attrs, "style")? {
            let (default, map_args) = ParseArgs::new()
                .meta((
                    conflicts((
                        ("default", path_only()).map(|_| FieldInit::Optional),
                        ("default", key_value::<Expr>()).map(FieldInit::OptionalWith),
                    ))
                    .optional(),
                    ("map", list(ParseArgs::new().args::<(Type, Path)>())).optional(),
                ))
                .parse_attr(attr)?
                .meta;
            (
                default.unwrap_or(FieldInit::AlwaysRequired),
                map_args.map(|a| a.args),
            )
        } else {
            (FieldInit::AlwaysRequired, None)
        };

        Ok(FieldInfo {
            origin: field,
            init: default,
            map: map_args,
        })
    }

    fn load_paths(&mut self, attrs: &[Attribute]) -> Result<()> {
        let ParseArgs {
            rest_args: path_exprs,
            meta: (add_path_all, derive_default),
            ..
        } = ParseArgs::new()
            .rest_args::<Vec<LitStr>>()
            .meta((("all", path_only()), ("derive_default", path_only())))
            .parse_concat_attrs(find_attr::all(attrs, "style"))?;

        if derive_default {
            self.derive_default = true;
            for (ident, field) in &self.all_fields {
                if let FieldInit::AlwaysRequired = &field.init {
                    return Err(Error::new_spanned(
                        ident,
                        "all fields should have default value if `derive_default` is specified",
                    ));
                }
            }
        }

        let mut unused_fields = HashMap::new();

        if add_path_all {
            Self::load_one_path(
                &self.all_fields,
                self.empty_path.from_tuple_order.clone(),
                &mut self.paths,
                &mut unused_fields,
            )?;
        }

        for path_expr in path_exprs {
            let raw_paths = path_expr.parse_with(style_path::parse)?;
            for path in raw_paths {
                Self::load_one_path(&self.all_fields, path, &mut self.paths, &mut unused_fields)?;
            }
        }

        Ok(())
    }

    fn load_one_path<'b>(
        all_fields: &'b HashMap<Member, FieldInfo<'a>>,
        mut path: Vec<Member>,
        path_vec: &mut Vec<PathDef>,
        unused_fields: &mut HashMap<&'b Member, &'b FieldInfo<'a>>,
    ) -> Result<()> {
        debug_assert!(unused_fields.is_empty());
        unused_fields.extend(all_fields.iter());

        for member in &path {
            if unused_fields.remove(member).is_none() {
                return Err(Error::new_spanned(
                    member,
                    format!(
                        "field `{}` {}",
                        display_member(member),
                        if all_fields.contains_key(member) {
                            "already used"
                        } else {
                            "not found"
                        }
                    ),
                ));
            }
        }

        let defined_len = path.len();
        path.reserve(unused_fields.len());

        for (unused, fi) in unused_fields.drain() {
            if let FieldInit::AlwaysRequired = &fi.init {
                return Err(Error::new_spanned(
                    unused,
                    format!(
                        "field `{}` should always be initialized, \
                                but is not initialized in at least one style path. \
                                if it is explicitly not required, \
                                use `#[style(default = ...)]` to specify a default value",
                        display_member(unused)
                    ),
                ));
            }
            path.push(unused.clone());
        }

        path_vec.push(PathDef {
            from_tuple_order: path,
            defined_len,
        });
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

        for path_def @ &PathDef {
            ref from_tuple_order,
            defined_len,
        } in &self.paths
        {
            let tuple_type = from_tuple_order.iter().take(defined_len).map(|member| {
                let field = &self.all_fields[member];
                match &field.map {
                    Some((ty, _)) => ty,
                    None => &field.origin.ty,
                }
            });
            let tuple_type = quote! { (#(#tuple_type,)*) };

            let body = self.compile_body(path_def);
            quote! {
                impl #impl_generics ::irisia::style::StyleFn<#tuple_type>
                    for #ident #ty_generics #where_clause
                {
                    fn from(__irisia_from: #tuple_type) -> Self {
                        #init_path #body
                    }
                }
            }
            .to_tokens(&mut tokens);
        }

        if self.derive_default {
            let body = self.compile_body(&self.empty_path);
            quote! {
                impl #impl_generics ::std::default::Default
                    for #ident #ty_generics #where_clause
                {
                    fn default() -> Self {
                        #init_path #body
                    }
                }
            }
            .to_tokens(&mut tokens);
        }

        tokens
    }

    fn compile_body(
        &self,
        &PathDef {
            ref from_tuple_order,
            defined_len,
        }: &PathDef,
    ) -> TokenStream {
        let defined_fields =
            (0..defined_len)
                .zip(from_tuple_order.iter())
                .map(|(index, member)| {
                    let index = Index::from(index);
                    let value = quote! { __irisia_from.#index };
                    self.compile_mapped_field(member, value)
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

    fn compile_mapped_field(&self, field: &Member, value: TokenStream) -> TokenStream {
        match &self.all_fields[field].map {
            Some((_, path)) => {
                quote! {
                    (#path)(#value)
                }
            }
            None => value,
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
            Self::AlwaysRequired => unreachable!(),
            Self::Optional => quote! {
                ::std::default::Default::default()
            },
            Self::OptionalWith(expr) => expr.to_token_stream(),
        }
    }
}
