use std::marker::PhantomData;

use definition::Definition;

use crate::{hook::watcher::Watcher, prim_element::Element};

use super::{Model, ModelCreateCtx, UnitModel, VModel, VNode};

pub mod definition;
pub mod property;

pub struct UseComponent<T, Cd, D> {
    _comp: PhantomData<T>,
    child_data: Cd,
    defs: D,
}

impl<T, D> UseComponent<T, ChildDataUndefined, D>
where
    T: Component,
    D: Definition,
{
    pub fn new(defs: D) -> Self {
        Self {
            _comp: PhantomData,
            child_data: ChildDataUndefined,
            defs,
        }
    }

    pub fn set_child_data<Cd>(self, child_data: Cd) -> UseComponent<T, ChildDataDefined<Cd>, D> {
        UseComponent {
            _comp: PhantomData,
            child_data: ChildDataDefined(child_data),
            defs: self.defs,
        }
    }
}

pub trait Component: 'static {
    fn create(self, watcher_list: &mut Vec<Watcher>) -> impl VNode + use<Self>;
}

impl<T, Cdmd, Cd, D> VModel<Cd> for UseComponent<T, Cdmd, D>
where
    Cdmd: ChildDataMaybeDefined<Cd> + Clone + 'static,
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
    model: Box<dyn UnitModel<()>>,
}

impl<D, Cdmd, Cd> Model<Cd> for UseComponentModel<D, Cdmd>
where
    Cdmd: ChildDataMaybeDefined<Cd>,
    Self: 'static,
{
    fn visit(&self, f: &mut dyn FnMut(Element, Cd)) {
        let (el, cd) = self.get_element();
        f(el, cd)
    }
}

impl<D, Cdmd, Cd> UnitModel<Cd> for UseComponentModel<D, Cdmd>
where
    Cdmd: ChildDataMaybeDefined<Cd>,
    Self: 'static,
{
    fn get_element(&self) -> (Element, Cd) {
        (self.model.get_element().0, self.child_data.get_child_data())
    }
}

// Child Data

#[derive(Clone)]
pub struct ChildDataDefined<T>(T);

#[derive(Clone, Copy)]
pub struct ChildDataUndefined;

pub trait ChildDataMaybeDefined<T> {
    fn get_child_data(&self) -> T;
}

impl<T: Clone> ChildDataMaybeDefined<T> for ChildDataDefined<T> {
    fn get_child_data(&self) -> T {
        self.0.clone()
    }
}

impl<T: Default> ChildDataMaybeDefined<T> for ChildDataUndefined {
    fn get_child_data(&self) -> T {
        T::default()
    }
}
