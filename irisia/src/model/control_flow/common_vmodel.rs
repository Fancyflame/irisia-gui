use std::any::Any;

use crate::prim_element::EMCreateCtx;

use crate::model::{Model, VModel};

pub struct BoxedModel(Box<dyn AnyModel>);

trait AnyModel: Any + Model {}
impl<T> AnyModel for T where T: Any + Model {}

pub trait CommonVModel {
    fn common_create(&self, ctx: &EMCreateCtx) -> BoxedModel;
    fn common_update(&self, storage: &mut BoxedModel, ctx: &EMCreateCtx);
}

impl<T> CommonVModel for T
where
    T: VModel,
{
    fn common_create(&self, ctx: &EMCreateCtx) -> BoxedModel {
        BoxedModel(Box::new(self.create(ctx)))
    }

    fn common_update(&self, storage: &mut BoxedModel, ctx: &EMCreateCtx) {
        let inner: &mut dyn AnyModel = &mut *storage.0;
        match (inner as &mut dyn Any).downcast_mut::<T::Storage>() {
            Some(inner_storage) => self.update(inner_storage, ctx),
            None => {
                const ERR_MSG: &str = "type mismatch detected when updating `BoxedModel`";

                if cfg!(debug_assertions) {
                    panic!("{ERR_MSG}");
                } else {
                    eprintln!("warning: {ERR_MSG}. create a new model instead.");
                }

                *storage = self.common_create(ctx);
            }
        }
    }
}

impl Model for BoxedModel {
    fn visit(&self, f: &mut dyn FnMut(crate::prim_element::Element)) {
        self.0.visit(f);
    }
}

impl VModel for dyn CommonVModel + '_ {
    type Storage = BoxedModel;

    fn create(&self, ctx: &EMCreateCtx) -> Self::Storage {
        self.common_create(ctx)
    }

    fn update(&self, storage: &mut Self::Storage, ctx: &EMCreateCtx) {
        self.common_update(storage, ctx);
    }
}
