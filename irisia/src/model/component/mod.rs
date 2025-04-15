use std::marker::PhantomData;

use definition::Definition;

use crate::prim_element::Element;

use super::{EleModel, Model, ModelCreateCtx, VModel, VNode};

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

pub trait Component: Default + 'static {
    type Created: 'static;

    fn create(self) -> (Self::Created, impl VNode);
}

impl<T, F, D> VModel for UseComponent<T, F, D>
where
    F: Fn(D::Value) -> T,
    T: Component,
    D: Definition,
{
    type Storage = UseComponentModel<T::Created, D::Storage>;

    fn create(&self, ctx: &ModelCreateCtx) -> Self::Storage {
        let (def_storages, def_values) = self.defs.create();
        let (component, vmodel) = T::create((self.create_fn)(def_values));

        let model = Box::new(vmodel.create(ctx));
        UseComponentModel {
            defs: def_storages,
            model,
            _component: component,
        }
    }

    fn update(&self, storage: &mut Self::Storage, _: &ModelCreateCtx) {
        self.defs.update(&mut storage.defs);
    }
}

pub struct UseComponentModel<T, D> {
    _component: T,
    defs: D,
    model: Box<dyn EleModel>,
}

impl<T, D> Model for UseComponentModel<T, D>
where
    Self: 'static,
{
    fn visit(&self, f: &mut dyn FnMut(crate::prim_element::Element)) {
        f(self.model.get_element())
    }
}

impl<T, D> EleModel for UseComponentModel<T, D>
where
    Self: 'static,
{
    fn get_element(&self) -> Element {
        self.model.get_element()
    }
}
