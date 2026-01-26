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

/// 通用虚模型：擦除生成的模型的类型，用于解决VModel不能做成trait object的问题
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
                log_error(storage.vmodel_name, type_name::<Self>());
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

fn log_error(expect_name: &str, got_name: &str) {
    error!(
        "different type detected when updating BoxedModel or BoxedUnitModel. \
        expected `{expect_name}`, but got `{got_name}`. creating a new one instead.",
    );
}
