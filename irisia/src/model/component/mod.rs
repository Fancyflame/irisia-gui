use std::{any::Any, marker::PhantomData};

use definition::Definition;

use crate::{
    hook::watcher::Watcher,
    model::{UnitAssertion, UnitVModel, VisitModelFn, control_flow::general::BoxedUnitModel},
};

use super::{Model, ModelCreateCtx, VModel};

pub mod definition;
pub mod property;

pub struct UseComponent<T, Cd, D> {
    _comp: PhantomData<T>,
    child_data: Cd,
    defs: D,
}

impl<T, D> UseComponent<T, ChildDataNone, D>
where
    T: Component,
    D: Definition,
{
    pub fn new(defs: D) -> Self {
        Self {
            _comp: PhantomData,
            child_data: ChildDataNone,
            defs,
        }
    }

    pub fn set_child_data<Cd>(self, child_data: Cd) -> UseComponent<T, ChildDataSome<Cd>, D> {
        UseComponent {
            _comp: PhantomData,
            child_data: ChildDataSome(child_data),
            defs: self.defs,
        }
    }
}

pub trait Component: 'static {
    fn create(self, watcher_list: &mut Vec<Watcher>) -> impl UnitVModel + use<Self>;
}

impl<T, Cdmd, D> VModel for UseComponent<T, Cdmd, D>
where
    Cdmd: ChildDataMaybeDefined + Clone + 'static,
    T: Component,
    D: Definition<Value = T>,
{
    type Storage = UseComponentModel<D::Storage, Cdmd>;

    fn create(&self, ctx: &ModelCreateCtx) -> Self::Storage {
        let (def_storages, def_values) = self.defs.create();
        let mut watcher_list = Vec::new();
        let vmodel = T::create(def_values, &mut watcher_list);

        let model = Box::new(vmodel.create(ctx));
        UseComponentModel {
            _watcher_list: watcher_list,
            defs: def_storages,
            child_data: self.child_data.clone(),
            model,
        }
    }

    fn update(&self, storage: &mut Self::Storage, _: &ModelCreateCtx) {
        self.defs.update(&mut storage.defs);
        storage.child_data = self.child_data.clone();
    }
}

pub struct UseComponentModel<D, Cdmd> {
    _watcher_list: Vec<Watcher>,
    defs: D,
    child_data: Cdmd,
    model: BoxedUnitModel,
}

impl<D, Cdmd> Model for UseComponentModel<D, Cdmd>
where
    Cdmd: ChildDataMaybeDefined,
    Self: 'static,
{
    fn visit_raw(&self, f: VisitModelFn) {
        self.model.visit_raw(|el, _| f(el, self.child_data.get()));
    }
}

impl<D, Cdmd> UnitAssertion for UseComponentModel<D, Cdmd> {}

// Child Data

#[derive(Clone)]
pub struct ChildDataSome<T>(T);

#[derive(Clone, Copy)]
pub struct ChildDataNone;

pub trait ChildDataMaybeDefined {
    fn get(&self) -> Option<&dyn Any>;
}

impl<T: 'static> ChildDataMaybeDefined for ChildDataSome<T> {
    fn get(&self) -> Option<&dyn Any> {
        Some(&self.0)
    }
}

impl ChildDataMaybeDefined for ChildDataNone {
    fn get(&self) -> Option<&dyn Any> {
        None
    }
}
