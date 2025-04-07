use definition::Definition;

use crate::hook::{reactive::Reactive, Signal};

use super::{
    control_flow::common_vmodel::{BoxedModel, CommonVModel},
    Model, VModel,
};

pub mod check_eq_helper;
pub mod definition;

pub struct UseComponent<F, D> {
    pub create_fn: F,
    pub defs: D,
}

pub trait Component {
    type Created: 'static;

    fn create(self) -> (Self::Created, Signal<dyn CommonVModel>);
}

impl<F, T, D> VModel for UseComponent<F, D>
where
    F: Fn(&D::Value) -> T,
    T: Component,
    D: Definition,
{
    type Storage = UseComponentModel<T::Created, D::Storage>;

    fn create(&self, ctx: &crate::prim_element::EMCreateCtx) -> Self::Storage {
        let (def_storages, def_values) = self.defs.create();
        let (component, vmodel) = (self.create_fn)(&def_values).create();

        let ctx = ctx.clone();
        let model = Reactive::builder(vmodel.create(&ctx))
            .dep(
                move |this, vm| {
                    VModel::update(vm, this, &ctx);
                },
                vmodel,
            )
            .build();

        UseComponentModel {
            _component: component,
            model,
            defs: def_storages,
        }
    }

    fn update(&self, storage: &mut Self::Storage, _: &crate::prim_element::EMCreateCtx) {
        self.defs.update(&mut storage.defs);
    }
}

pub struct UseComponentModel<T, D> {
    _component: T,
    defs: D,
    model: Reactive<BoxedModel>,
}

impl<T, D> Model for UseComponentModel<T, D>
where
    T: 'static,
    D: 'static,
{
    fn visit(&self, f: &mut dyn FnMut(crate::prim_element::Element)) {
        self.model.read().visit(f);
    }
}
