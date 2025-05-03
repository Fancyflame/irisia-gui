use std::marker::PhantomData;

use definition::Definition;

use crate::{hook::watcher::WatcherList, prim_element::Element};

use super::{EleModel, GetParentPropsFn, Model, ModelCreateCtx, VModel, VNode};

pub mod definition;
pub mod direct_assign_helper;
pub mod proxy_signal_helper;

pub struct UseComponent<T, Pp, F, D> {
    _comp: PhantomData<T>,
    parent_props: Pp,
    create_fn: F,
    defs: D,
}

impl<Pp, T, F, D> UseComponent<T, Pp, F, D>
where
    T: Component,
    F: Fn(D::Value) -> T,
    D: Definition,
{
    pub fn new(parent_props: Pp, create_fn: F, defs: D) -> Self {
        Self {
            parent_props,
            _comp: PhantomData,
            create_fn,
            defs,
        }
    }
}

pub type GetChildProps<T> = <T as Component>::ChildProps;

pub trait Component: Default + 'static {
    type ChildProps: Default;

    fn create(self, watcher_list: &mut WatcherList) -> impl ComponentVNode;
}

impl<T, Pp, F, D> VModel for UseComponent<T, Pp, F, D>
where
    F: Fn(D::Value) -> T,
    T: Component,
    D: Definition,
{
    type Storage = UseComponentModel<D::Storage>;
    type ParentProps = Pp;

    fn get_parent_props(&self, f: GetParentPropsFn<Self::ParentProps>) {
        f(&self.parent_props)
    }

    fn create(&self, ctx: &ModelCreateCtx) -> Self::Storage {
        let (def_storages, def_values) = self.defs.create();
        let mut watcher_list = WatcherList::new();
        let vmodel = T::create((self.create_fn)(def_values), &mut watcher_list);

        let model = Box::new(vmodel.create(ctx));
        UseComponentModel {
            _watcher_list: watcher_list,
            defs: def_storages,
            model,
        }
    }

    fn update(&self, storage: &mut Self::Storage, _: &ModelCreateCtx) {
        self.defs.update(&mut storage.defs);
    }
}

pub struct UseComponentModel<D> {
    _watcher_list: WatcherList,
    defs: D,
    model: Box<dyn EleModel>,
}

impl<D> Model for UseComponentModel<D>
where
    Self: 'static,
{
    fn visit(&self, f: &mut dyn FnMut(crate::prim_element::Element)) {
        f(self.model.get_element())
    }
}

impl<D> EleModel for UseComponentModel<D>
where
    Self: 'static,
{
    fn get_element(&self) -> Element {
        self.model.get_element()
    }
}

pub fn assert_vnode<T>(n: T) -> T
where
    T: VNode,
{
    n
}

pub trait ComponentVNode: VNode<ParentProps = ()> + 'static {}
impl<T: VNode<ParentProps = ()> + 'static> ComponentVNode for T {}
