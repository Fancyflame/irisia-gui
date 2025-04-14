use prim_element::BlockModel;

use crate::{
    hook::reactive::WeakReactive,
    prim_element::{EMCreateCtx, Element, GetElement},
};

pub mod component;
pub mod control_flow;
pub mod prim_element;

pub trait VModel {
    type Storage: Model;

    fn create(&self, ctx: &ModelCreateCtx) -> Self::Storage;
    fn update(&self, storage: &mut Self::Storage, ctx: &ModelCreateCtx);
}

pub trait Model: 'static {
    fn visit(&self, f: &mut dyn FnMut(Element));
}

pub trait VNode: VModel<Storage: GetElement> {}

impl<T> VNode for T where T: VModel<Storage: GetElement> {}

#[derive(Clone)]
pub struct ModelCreateCtx {
    el_ctx: EMCreateCtx,
    parent: WeakReactive<BlockModel>,
}
