use std::marker::PhantomData;

use definition::Definition;

use crate::{hook::Signal, prim_element::EMCreateCtx};

use super::{
    control_flow::{
        common_vmodel::{BoxedModel, CommonVModel},
        signal::SignalModel,
    },
    Model, VModel,
};

pub mod check_eq_helper;
pub mod definition;

pub struct UseComponent<T, F, D> {
    pub _comp: PhantomData<T>,
    pub create_fn: F,
    pub defs: D,
}

pub trait Component: 'static {
    type Props;

    fn create(props: Self::Props) -> Self;
    fn render(&self) -> Signal<dyn CommonVModel>;
}

impl<T, F, D> VModel for UseComponent<T, F, D>
where
    F: Fn(&D::Value) -> T::Props,
    T: Component,
    D: Definition,
{
    type Storage = UseComponentModel<T, D::Storage>;

    fn create(&self, ctx: &EMCreateCtx) -> Self::Storage {
        let (def_storages, def_values) = self.defs.create();
        let component = T::create((self.create_fn)(&def_values));

        UseComponentModel {
            defs: def_storages,
            model: component.render().create(ctx),
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
    model: SignalModel<BoxedModel>,
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
