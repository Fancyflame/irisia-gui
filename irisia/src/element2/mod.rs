use std::{cell::RefCell, rc::Rc};

use hooks::{HookStorage, UseHook};

use crate::model2::VModel;

pub mod hooks;

/// Element is a thing can draw itself on the given canvas,
/// according to its properties, styles and given drawing region.
/// This trait is close to the native rendering, if you are not a
/// component maker, please using exist elements or macros to
/// customize one.
pub trait Component {
    type OutVModel: VModel + 'static;
    fn run(self, use_hook: UseHook) -> Self::OutVModel;
}

pub struct CompRuntime<T> {
    inner: Rc<RefCell<Inner<T>>>,
}

struct Inner<T> {
    model: T,
    hooks: HookStorage,
}

pub struct Unit<Pr, Cd> {
    pub props: Pr,
    pub child_data: Cd,
}

impl<T, Cd> VModel for Unit<T, Cd>
where
    T: Component,
{
    type Storage = CompRuntime<<T::OutVModel as VModel>::Storage>;
    fn create(self, ctx: &crate::el_model::EMCreateCtx) -> Self::Storage {
        let (hooks, model) = HookStorage::new(|use_hook| self.props.run(use_hook).create(ctx));
        let inner = Inner { hooks, model };

        CompRuntime {
            inner: Rc::new(RefCell::new(inner)),
        }
    }
    fn update(self, storage: &mut Self::Storage, ctx: &crate::el_model::EMCreateCtx) {
        let mut storage = storage.inner.borrow_mut();
        let inner = &mut *storage;

        self.props
            .run(inner.hooks.call())
            .update(&mut inner.model, ctx);
    }
}
