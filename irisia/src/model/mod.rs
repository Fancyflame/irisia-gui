use map_parent_props::MapParentProps;
use prim::BlockModel;

use crate::{
    hook::reactive::WeakReactive,
    prim_element::{EMCreateCtx, Element},
};

pub mod component;
pub mod control_flow;
pub mod map_parent_props;
pub mod prim;

pub trait VModel {
    type Storage: Model;
    type ParentProps;

    fn get_parent_props(&self, f: GetParentPropsFn<Self::ParentProps>);
    fn create(&self, ctx: &ModelCreateCtx) -> Self::Storage;
    fn update(&self, storage: &mut Self::Storage, ctx: &ModelCreateCtx);

    // Provided

    fn map_parent_props<F, U>(self, f: F) -> MapParentProps<F, Self>
    where
        Self: Sized,
        F: Fn(&Self::ParentProps) -> &U,
    {
        MapParentProps {
            src_vmodel: self,
            map: f,
        }
    }

    fn clear_parent_props(self) -> MapParentProps<(), Self>
    where
        Self: Sized,
    {
        MapParentProps {
            map: (),
            src_vmodel: self,
        }
    }
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

pub type GetParentPropsFn<'a, Pp> = &'a mut dyn FnMut(&Pp);
