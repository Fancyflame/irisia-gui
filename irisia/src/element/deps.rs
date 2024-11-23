use crate::{
    application::content::GlobalContent,
    hook::{consumer::Consumer, ProviderObject},
};

pub struct NeedInit;

pub trait EmptyProps {
    type AsEmpty;
    fn empty_props() -> Self::AsEmpty;
}
