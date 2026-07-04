use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::{
    Handle,
    hook::{Signal, watcher::Watcher},
    model::{Model, ModelCreateCtx, VModel, VisitModelFn},
};

impl<T> VModel for Signal<T>
where
    T: VModel + ?Sized + 'static,
{
    type Storage = SignalModel<T::Storage>;

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
    let mut watcher_list = Vec::new();

    watcher_list.push(Watcher::watch(vmodel.clone(), {
        let model = model.clone();
        move |vmodel: &T| {
            vmodel.update(&mut model.borrow_mut(), &ctx);
            if let Some(parent) = ctx.parent.as_ref().and_then(Weak::upgrade) {
                parent.borrow_mut().submit_children();
            }
        }
    }));

    SignalModel {
        vmodel_addr: vmodel.addr(),
        _watcher_list: watcher_list,
        model: Some(model),
    }
}

pub struct SignalModel<T> {
    vmodel_addr: *const (),
    _watcher_list: Vec<Watcher>,
    model: Option<Handle<T>>,
}

impl<T> Model for SignalModel<T>
where
    T: Model,
{
    fn visit_raw(&self, f: VisitModelFn) {
        self.model.as_ref().unwrap().borrow().visit_raw(f);
    }
}
