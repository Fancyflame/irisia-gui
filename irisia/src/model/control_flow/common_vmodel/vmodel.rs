use crate::{
    model::{Model, ModelCreateCtx, VModel},
    prim_element::Element,
};

use std::any::{Any, type_name};

pub struct BoxedModel<Cd>(Box<dyn AnyModel<Cd>>);

trait AnyModel<Cd>: Any + Model<Cd> {
    fn type_name(&self) -> &'static str;
}

impl<Cd, T> AnyModel<Cd> for T
where
    T: Any + Model<Cd>,
{
    fn type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }
}

pub trait CommonVModel<Cd> {
    fn common_create(&self, ctx: &ModelCreateCtx) -> BoxedModel<Cd>;
    fn common_update(&self, storage: &mut BoxedModel<Cd>, ctx: &ModelCreateCtx);
}

impl<Cd: 'static, T> CommonVModel<Cd> for T
where
    T: VModel<Cd>,
{
    fn common_create(&self, ctx: &ModelCreateCtx) -> BoxedModel<Cd> {
        BoxedModel(Box::new(self.create(ctx)))
    }

    fn common_update(&self, storage: &mut BoxedModel<Cd>, ctx: &ModelCreateCtx) {
        let expected_type_name = storage.0.type_name();
        let inner: &mut dyn AnyModel<Cd> = &mut *storage.0;

        match (inner as &mut dyn Any).downcast_mut::<T::Storage>() {
            Some(inner_storage) => self.update(inner_storage, ctx),
            None => {
                const ERR_MSG: &str = "type mismatch detected when updating `BoxedModel`";

                if cfg!(debug_assertions) {
                    panic!(
                        "{ERR_MSG}. \nexpected `{expected_type_name}`\nfound `{}`",
                        type_name::<T::Storage>()
                    );
                } else {
                    eprintln!("warning: {ERR_MSG}. create a new model instead.");
                }

                *storage = self.common_create(ctx);
            }
        }
    }
}

impl<Cd: 'static> Model<Cd> for BoxedModel<Cd> {
    fn visit(&self, f: &mut dyn FnMut(Element, Cd)) {
        self.0.visit(f);
    }
}

impl<Cd: 'static> VModel<Cd> for dyn CommonVModel<Cd> + '_ {
    type Storage = BoxedModel<Cd>;

    fn create(&self, ctx: &ModelCreateCtx) -> Self::Storage {
        self.common_create(ctx)
    }

    fn update(&self, storage: &mut Self::Storage, ctx: &ModelCreateCtx) {
        self.common_update(storage, ctx);
    }
}
