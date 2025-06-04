use std::marker::PhantomData;

use definition::Definition;

use crate::{hook::watcher::WatcherList, prim_element::Element};

use super::{EleModel, Model, ModelCreateCtx, VModel, VNode};

pub mod definition;
pub mod direct_assign_helper;
pub mod proxy_signal_helper;

pub struct UseComponent<T, Cd, F, D> {
    _comp: PhantomData<T>,
    child_data: Cd,
    create_fn: F,
    defs: D,
}

impl<T, F, D> UseComponent<T, ChildDataUndefined, F, D>
where
    T: Component,
    F: Fn(D::Value) -> T,
    D: Definition,
{
    pub fn new(create_fn: F, defs: D) -> Self {
        Self {
            _comp: PhantomData,
            child_data: ChildDataUndefined,
            create_fn,
            defs,
        }
    }

    pub fn set_child_data<Cd>(self, child_data: Cd) -> UseComponent<T, ChildDataDefined<Cd>, F, D> {
        UseComponent {
            _comp: PhantomData,
            child_data: ChildDataDefined(child_data),
            create_fn: self.create_fn,
            defs: self.defs,
        }
    }
}

impl<Cd, T, F, D> UseComponent<T, ChildDataDefined<Cd>, F, D>
where
    T: Component,
    F: Fn(D::Value) -> T,
    D: Definition,
{
    pub fn with_child_data(child_data: Cd, create_fn: F, defs: D) -> Self {
        Self {
            _comp: PhantomData,
            child_data: ChildDataDefined(child_data),
            create_fn,
            defs,
        }
    }
}

pub trait Component: Default + 'static {
    fn create(self, watcher_list: &mut WatcherList) -> impl VNode<()> + use<Self>;
}

impl<T, Cdmd, Cd, F, D> VModel<Cd> for UseComponent<T, Cdmd, F, D>
where
    Cdmd: ChildDataMaybeDefined<Cd> + Clone + 'static,
    F: Fn(D::Value) -> T,
    T: Component,
    D: Definition,
{
    type Storage = UseComponentModel<D::Storage, Cdmd>;

    fn create(&self, ctx: &ModelCreateCtx) -> Self::Storage {
        let (def_storages, def_values) = self.defs.create();
        let mut watcher_list = WatcherList::new();
        let vmodel = T::create((self.create_fn)(def_values), &mut watcher_list);

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
    }
}

pub struct UseComponentModel<D, Cdmd> {
    _watcher_list: WatcherList,
    defs: D,
    child_data: Cdmd,
    model: Box<dyn EleModel<()>>,
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

impl<D, Cdmd, Cd> EleModel<Cd> for UseComponentModel<D, Cdmd>
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
