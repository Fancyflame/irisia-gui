use prim::BlockModel;

use crate::{
    hook::reactive::WeakReactive,
    prim_element::{EMCreateCtx, Element},
};

pub mod component;
pub mod control_flow;
pub mod prim;

pub trait VModel {
    type Storage: Model;

    fn create(&self, ctx: &ModelCreateCtx) -> Self::Storage;
    fn update(&self, storage: &mut Self::Storage, ctx: &ModelCreateCtx);
}

pub trait Model: 'static {
    fn visit(&self, f: &mut dyn FnMut(Element));
}
#[derive(Clone)]
pub struct ModelCreateCtx {
    el_ctx: EMCreateCtx,
    parent: Option<WeakReactive<BlockModel>>,
}

impl ModelCreateCtx {
    pub(crate) fn create_as_root(ctx: EMCreateCtx) -> Self {
        Self {
            el_ctx: ctx,
            parent: None,
        }
    }
}

/// VModel provides guaranteed only 1 element
pub trait VNode: VModel<Storage: EleModel> {}
impl<T> VNode for T where T: VModel<Storage: EleModel> {}

pub trait EleModel: Model {
    fn get_element(&self) -> Element;
}

pub trait GetParentProps<T> {
    fn get_parent_props(&self, dst: &mut Vec<T>);
}
