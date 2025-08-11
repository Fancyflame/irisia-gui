use crate::{
    model::{EleModel, Model, ModelCreateCtx, VModel, VNode},
    prim_element::Element,
};

use std::any::Any;

pub struct BoxedNode<Cd>(Box<dyn AnyNode<Cd>>);

trait AnyNode<Cd>: Any + EleModel<Cd> {}

impl<T, Cd> AnyNode<Cd> for T where T: Any + EleModel<Cd> {}

pub trait CommonVNode<Cd> {
    fn common_create_node(&self, ctx: &ModelCreateCtx) -> BoxedNode<Cd>;
    fn common_update_node(&self, storage: &mut BoxedNode<Cd>, ctx: &ModelCreateCtx);
}

impl<Cd, T> CommonVNode<Cd> for T
where
    T: VNode<Cd>,
{
    fn common_create_node(&self, ctx: &ModelCreateCtx) -> BoxedNode<Cd> {
        BoxedNode(Box::new(self.create(ctx)))
    }

    fn common_update_node(&self, storage: &mut BoxedNode<Cd>, ctx: &ModelCreateCtx) {
        let inner: &mut dyn AnyNode<Cd> = &mut *storage.0;
        match (inner as &mut dyn Any).downcast_mut::<T::Storage>() {
            Some(inner_storage) => self.update(inner_storage, ctx),
            None => {
                const ERR_MSG: &str = "type mismatch detected when updating `BoxedNode`";

                if cfg!(debug_assertions) {
                    panic!("{ERR_MSG}");
                } else {
                    eprintln!("warning: {ERR_MSG}. create a new model instead.");
                }

                *storage = self.common_create_node(ctx);
            }
        }
    }
}

impl<Cd: 'static> Model<Cd> for BoxedNode<Cd> {
    fn visit(&self, f: &mut dyn FnMut(Element, Cd)) {
        self.0.visit(f);
    }
}

impl<Cd: 'static> EleModel<Cd> for BoxedNode<Cd> {
    fn get_element(&self) -> (Element, Cd) {
        self.0.get_element()
    }
}

impl<Cd: 'static> VModel<Cd> for dyn CommonVNode<Cd> + '_ {
    type Storage = BoxedNode<Cd>;

    fn create(&self, ctx: &ModelCreateCtx) -> Self::Storage {
        self.common_create_node(ctx)
    }

    fn update(&self, storage: &mut Self::Storage, ctx: &ModelCreateCtx) {
        self.common_update_node(storage, ctx);
    }
}
