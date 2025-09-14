const_quote! {
    pub const PATH_COMPONENT = { irisia::model::component };
    pub const COERCE_HOOK = { irisia::coerce_hook! };
    pub const PATH_PROPERTY = { #PATH_COMPONENT::property };
    pub const PATH_RC = { ::std::rc::Rc };
    pub const PATH_OPTION = { ::core::option::Option };
    pub const TRAIT_DEFAULT = { ::std::default::Default };
}
