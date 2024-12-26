use std::{any::Any, cell::RefCell, rc::Rc};

use hooks::{HookStorage, UseHook};
use schedule_rerun::ScheduleRerun;

use crate::{
    el_model::EMCreateCtx,
    model2::{VModel, VNode},
    prim_element::GetElement,
};

pub mod hooks;
pub mod schedule_rerun;

/// Element is a thing can draw itself on the given canvas,
/// according to its properties, styles and given drawing region.
/// This trait is close to the native rendering, if you are not a
/// component maker, please using exist elements or macros to
/// customize one.
pub trait Component: 'static {
    fn run(&self, use_hook: UseHook) -> impl VNode;
}

pub struct CompModel<Pr> {
    inner: Rc<RefCell<CompModelInner<Pr>>>,
}

struct CompModelInner<Pr> {
    needs_rerun: bool,
    props: Rc<Pr>,
    storage_model: Box<dyn StorageModel>,
    hooks: HookStorage,
    ctx: EMCreateCtx,
}

trait StorageModel: GetElement {
    fn as_any(&mut self) -> &mut dyn Any;
}

impl<T> StorageModel for T
where
    T: GetElement + Any,
{
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl<Pr: Component> CompModelInner<Pr> {
    fn rerun(&mut self) {
        self.props.run(self.hooks.call()).update(
            self.storage_model.as_any().downcast_mut().unwrap(),
            &self.ctx,
        );
        self.needs_rerun = false;
    }
}

pub struct Comp<Pr, Cd> {
    pub props: Rc<Pr>,
    pub child_data: Cd,
}

impl<T, Cd> VModel for Comp<T, Cd>
where
    T: Component,
    Cd: 'static,
{
    type Storage = CompModel<T>;
    fn create(&self, ctx: &EMCreateCtx) -> Self::Storage {
        let inner = Rc::new_cyclic(|weak| {
            let (hooks, storage_model) =
                HookStorage::new(ScheduleRerun(weak.clone() as _), |use_hook| {
                    let storage = self.props.run(use_hook).create(ctx);
                    Box::new(storage) as Box<dyn StorageModel>
                });

            RefCell::new(CompModelInner {
                needs_rerun: false,
                hooks,
                storage_model,
                props: self.props.clone(),
                ctx: ctx.clone(),
            })
        });

        CompModel { inner }
    }
    fn update(&self, storage: &mut Self::Storage, _: &EMCreateCtx) {
        let mut storage = storage.inner.borrow_mut();
        let inner = &mut *storage;

        inner.props = self.props.clone();
        inner.rerun();
    }
}

impl<Pr: 'static> GetElement for CompModel<Pr> {
    fn get_element(&self) -> crate::prim_element::Element {
        self.inner.borrow().storage_model.get_element()
    }
}
