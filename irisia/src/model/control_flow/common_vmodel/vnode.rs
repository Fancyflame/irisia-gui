use crate::model::{EleModel, Model, ModelCreateCtx, VModel, VNode};

use std::any::Any;

pub struct BoxedNode(Box<dyn AnyNode>);

trait AnyNode: Any + EleModel {}

impl<T> AnyNode for T where T: Any + EleModel {}

pub trait CommonVNode {
    fn common_create_node(&self, ctx: &ModelCreateCtx) -> BoxedNode;
    fn common_update_node(&self, storage: &mut BoxedNode, ctx: &ModelCreateCtx);
}

impl<T> CommonVNode for T
where
    T: VNode,
{
    fn common_create_node(&self, ctx: &ModelCreateCtx) -> BoxedNode {
        BoxedNode(Box::new(self.create(ctx)))
    }

    fn common_update_node(&self, storage: &mut BoxedNode, ctx: &ModelCreateCtx) {
        let inner: &mut dyn AnyNode = &mut *storage.0;
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

impl Model for BoxedNode {
    fn visit(&self, f: &mut dyn FnMut(crate::prim_element::Element)) {
        self.0.visit(f);
    }
}

impl EleModel for BoxedNode {
    fn get_element(&self) -> crate::prim_element::Element {
        self.0.get_element()
    }
}

impl VModel for dyn CommonVNode + '_ {
    type Storage = BoxedNode;

    fn create(&self, ctx: &ModelCreateCtx) -> Self::Storage {
        self.common_create_node(ctx)
    }

    fn update(&self, storage: &mut Self::Storage, ctx: &ModelCreateCtx) {
        self.common_update_node(storage, ctx);
    }
}
