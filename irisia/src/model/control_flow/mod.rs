use crate::{
    prim_element::EMCreateCtx,
    prim_element::{Element, GetElement},
};

pub mod branch;
pub mod repeat;
pub mod slot;
pub mod tuple;

pub trait VModel {
    type Storage: Model;

    fn update(self, storage: &mut Self::Storage, ctx: &EMCreateCtx);
    fn create(self, ctx: &EMCreateCtx) -> Self::Storage;
}

pub trait Model: 'static {
    fn visit(&self, f: &mut dyn FnMut(Element));
}

pub trait VNode: VModel<Storage: GetElement> {}

impl<T> VNode for T where T: VModel<Storage: GetElement> {}
