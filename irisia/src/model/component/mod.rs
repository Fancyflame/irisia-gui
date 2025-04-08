use std::marker::PhantomData;

use definition::Definition;

use crate::prim_element::EMCreateCtx;

use super::{
    control_flow::common_vmodel::{BoxedModel, CommonVModel},
    Model, VModel,
};

pub mod definition;
pub mod proxy_signal_helper;

pub struct UseComponent<T, F, D> {
    _comp: PhantomData<T>,
    create_fn: F,
    defs: D,
}

impl<T, F, D> UseComponent<T, F, D>
where
    T: Component,
    F: Fn(D::Value) -> T,
    D: Definition,
{
    pub fn new(create_fn: F, defs: D) -> Self {
        Self {
            _comp: PhantomData,
            create_fn,
            defs,
        }
    }
}

pub trait Component: Sized + 'static {
    type Created: 'static;

    fn create(self) -> (Self::Created, impl VModel);
}

impl<T, F, D> VModel for UseComponent<T, F, D>
where
    F: Fn(D::Value) -> T,
    T: Component,
    D: Definition,
{
    type Storage = UseComponentModel<T::Created, D::Storage>;

    fn create(&self, ctx: &EMCreateCtx) -> Self::Storage {
        let (def_storages, def_values) = self.defs.create();
        let (component, vmodel) = T::create((self.create_fn)(def_values));

        let model = vmodel.common_create(ctx);
        UseComponentModel {
            defs: def_storages,
            model,
            _component: component,
        }
    }

    fn update(&self, storage: &mut Self::Storage, _: &EMCreateCtx) {
        self.defs.update(&mut storage.defs);
    }
}

pub struct UseComponentModel<T, D> {
    _component: T,
    defs: D,
    model: BoxedModel,
}

impl<T, D> Model for UseComponentModel<T, D>
where
    T: 'static,
    D: 'static,
{
    fn visit(&self, f: &mut dyn FnMut(crate::prim_element::Element)) {
        self.model.visit(f);
    }
}
