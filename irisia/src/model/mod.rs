use std::any::Any;

use crate::{
    WeakHandle,
    model::control_flow::general::GeneralVModel,
    prim_element::{EMCreateCtx, Element},
};

pub mod control_flow;

type VisitModelFn<'a> = &'a mut (dyn FnMut(Element, Option<&dyn Any>) + 'a);

pub trait VModel {
    type Storage: Model;

    fn create(&self, ctx: &ModelCreateCtx) -> Self::Storage;
    fn update(&self, storage: &mut Self::Storage, ctx: &ModelCreateCtx);

    fn as_general(&self) -> &dyn GeneralVModel
    where
        Self: Sized,
    {
        self
    }
}

pub trait Model: 'static {
    fn visit_raw(&self, f: VisitModelFn);
}

pub trait ModelExt: Model {
    fn visit<F, Data>(&self, mut f: F)
    where
        F: FnMut(Element, Option<&Data>),
        Data: 'static,
    {
        self.visit_raw(&mut |el, optioned_data| {
            f(el, optioned_data.and_then(|data| data.downcast_ref()))
        });
    }
}
impl<T: Model + ?Sized> ModelExt for T {}

#[derive(Clone)]
pub struct ModelCreateCtx {
    el_ctx: EMCreateCtx,
}

impl ModelCreateCtx {
    pub(crate) fn create_as_root(ctx: EMCreateCtx) -> Self {
        Self { el_ctx: ctx }
    }
}
