use crate::model::{Model, ModelCreateCtx, VModel, VisitModelFn};
use irisia_log::error;
use std::any::{Any, type_name};

pub use unit::*;

mod unit;

trait BoxedModelInner: Model + Any {}
impl<T: Model + Any> BoxedModelInner for T {}

pub struct BoxedModel {
    storage: Box<dyn BoxedModelInner>,
    vmodel_name: &'static str,
}

pub trait GeneralVModel {
    fn dyn_create(&self, ctx: &ModelCreateCtx) -> BoxedModel;
    fn dyn_update(&self, storage: &mut BoxedModel, ctx: &ModelCreateCtx);
}

impl<T> GeneralVModel for T
where
    T: VModel + ?Sized,
{
    fn dyn_create(&self, ctx: &ModelCreateCtx) -> BoxedModel {
        BoxedModel {
            storage: Box::new(self.create(ctx)),
            vmodel_name: type_name::<Self>(),
        }
    }
    fn dyn_update(&self, storage: &mut BoxedModel, ctx: &ModelCreateCtx) {
        let inner: &mut dyn Any = storage.storage.as_mut();
        match inner.downcast_mut::<T::Storage>() {
            Some(inner_storage) => self.update(inner_storage, ctx),
            None => {
                log_error("BoxedModel", storage.vmodel_name, type_name::<Self>());
                *storage = self.dyn_create(ctx);
            }
        }
    }
}

impl VModel for dyn GeneralVModel {
    type Storage = BoxedModel;
    fn create(&self, ctx: &ModelCreateCtx) -> Self::Storage {
        self.dyn_create(ctx)
    }
    fn update(&self, storage: &mut Self::Storage, ctx: &ModelCreateCtx) {
        self.dyn_update(storage, ctx);
    }
}

impl Model for BoxedModel {
    fn visit_raw(&self, f: VisitModelFn) {
        self.storage.visit_raw(f);
    }
}

fn log_error(box_name: &str, expect_name: &str, got_name: &str) {
    error!(
        "type mismatch detected when updating `{box_name}`. \
        expected `{expect_name}`, but got `{got_name}`. creating a new one instead.",
    );
}
