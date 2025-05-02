use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::{
    hook::{
        watcher::{WatcherGuard, WatcherList},
        Signal,
    },
    model::{EleModel, GetParentPropsFn, Model, ModelCreateCtx, VModel},
    Handle,
};

impl<T> VModel for Signal<T>
where
    T: VModel + ?Sized + 'static,
{
    type ParentProps = T::ParentProps;
    type Storage = SignalModel<T::Storage>;

    fn get_parent_props(&self, f: GetParentPropsFn<Self::ParentProps>) {
        self.read().get_parent_props(f);
    }

    fn create(&self, ctx: &ModelCreateCtx) -> Self::Storage {
        make_model(self, Rc::new(RefCell::new(self.read().create(ctx))), ctx)
    }

    fn update(&self, storage: &mut Self::Storage, ctx: &ModelCreateCtx) {
        if storage.vmodel_addr == self.addr() {
            return;
        }

        *storage = make_model(self, storage.model.take().unwrap(), ctx);
    }
}

fn make_model<T>(
    vmodel: &Signal<T>,
    init_state: Rc<RefCell<T::Storage>>,
    ctx: &ModelCreateCtx,
) -> SignalModel<T::Storage>
where
    T: VModel + ?Sized + 'static,
{
    let ctx = ctx.clone();
    let model = init_state.clone();
    let mut watcher_list = WatcherList::new();

    watcher_list.watch(
        {
            let model = model.clone();
            move |vmodel: &T| {
                vmodel.update(&mut model.borrow_mut(), &ctx);
                if let Some(parent) = ctx.parent.as_ref().and_then(Weak::upgrade) {
                    parent.borrow_mut().submit_children();
                }
            }
        },
        vmodel.clone(),
    );

    SignalModel {
        vmodel_addr: vmodel.addr(),
        _watcher_list: watcher_list,
        model: Some(model),
    }
}

pub struct SignalModel<T> {
    vmodel_addr: *const (),
    _watcher_list: WatcherList,
    model: Option<Handle<T>>,
}

impl<T> Model for SignalModel<T>
where
    T: Model,
{
    fn visit(&self, f: &mut dyn FnMut(crate::prim_element::Element)) {
        self.model.as_ref().unwrap().borrow().visit(f);
    }
}

impl<T> EleModel for SignalModel<T>
where
    T: EleModel,
{
    fn get_element(&self) -> crate::prim_element::Element {
        self.model.as_ref().unwrap().borrow().get_element()
    }
}
