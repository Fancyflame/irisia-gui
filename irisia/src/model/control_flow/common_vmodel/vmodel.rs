use crate::model::{GetParentPropsFn, Model, ModelCreateCtx, VModel};

use std::any::Any;

pub struct BoxedModel(Box<dyn AnyModel>);

trait AnyModel: Any + Model {}

impl<T> AnyModel for T where T: Any + Model {}

pub trait CommonVModel {
    type CommonParentProps;

    fn common_get_parent_props(&self, f: GetParentPropsFn<Self::CommonParentProps>);
    fn common_create(&self, ctx: &ModelCreateCtx) -> BoxedModel;
    fn common_update(&self, storage: &mut BoxedModel, ctx: &ModelCreateCtx);
}

impl<T> CommonVModel for T
where
    T: VModel,
{
    type CommonParentProps = T::ParentProps;

    fn common_get_parent_props(&self, f: GetParentPropsFn<Self::CommonParentProps>) {
        self.get_parent_props(f);
    }

    fn common_create(&self, ctx: &ModelCreateCtx) -> BoxedModel {
        BoxedModel(Box::new(self.create(ctx)))
    }

    fn common_update(&self, storage: &mut BoxedModel, ctx: &ModelCreateCtx) {
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

impl<Pp> VModel for dyn CommonVModel<CommonParentProps = Pp> + '_ {
    type Storage = BoxedModel;
    type ParentProps = Pp;

    fn get_parent_props(&self, f: GetParentPropsFn<Self::ParentProps>) {
        self.common_get_parent_props(f);
    }

    fn create(&self, ctx: &ModelCreateCtx) -> Self::Storage {
        self.common_create(ctx)
    }

    fn update(&self, storage: &mut Self::Storage, ctx: &ModelCreateCtx) {
        self.common_update(storage, ctx);
    }
}
