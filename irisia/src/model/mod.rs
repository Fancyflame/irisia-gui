use std::any::Any;

use crate::{
    WeakHandle,
    model::prim::SubmitChildren,
    prim_element::{EMCreateCtx, Element},
};

pub use style::UseStyle;
pub use unit::*;

pub mod component;
pub mod control_flow;
pub mod prim;
pub mod style;
mod unit;

type VisitModelFn<'a> = &'a mut (dyn FnMut(Element, Option<&dyn Any>) + 'a);

pub trait VModel {
    type Storage: Model;

    fn create(&self, ctx: &ModelCreateCtx) -> Self::Storage;
    fn update(&self, storage: &mut Self::Storage, ctx: &ModelCreateCtx);
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
    parent: Option<WeakHandle<dyn SubmitChildren>>,
}

impl ModelCreateCtx {
    pub(crate) fn create_as_root(ctx: EMCreateCtx) -> Self {
        Self {
            el_ctx: ctx,
            parent: None,
        }
    }
}
