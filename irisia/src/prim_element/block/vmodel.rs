use std::{cell::RefCell, rc::Rc};

use crate::{
    model2::{Model, VModel},
    prim_element::{Element, GetElement, Handle},
};

use super::{LayoutFn, RenderBlock};

#[derive(PartialEq, Clone)]
pub struct Block<T> {
    pub children: T,
    pub layout_fn: LayoutFn,
}

pub struct BlockModel<T> {
    storage: T,
    node: Handle<RenderBlock>,
}

impl<T> VModel for Block<T>
where
    T: VModel,
{
    type Storage = BlockModel<T::Storage>;
    fn create(&self, ctx: &crate::el_model::EMCreateCtx) -> Self::Storage {
        let children = self.children.create(ctx);
        let mut element_list = vec![];
        children.visit(&mut |el| element_list.push(el.clone()));

        BlockModel {
            storage: children,
            node: Rc::new(RefCell::new(RenderBlock {
                prev_draw_region: None,
                need_relayout: true,
                layout_fn: self.layout_fn,
                children: element_list,
                children_draw_regions: vec![],
            })),
        }
    }
    fn update(&self, storage: &mut Self::Storage, ctx: &crate::el_model::EMCreateCtx) {
        let mut node = storage.node.borrow_mut();

        node.layout_fn = self.layout_fn;
        self.children.update(&mut storage.storage, ctx);
        node.children.clear();
        storage
            .storage
            .visit(&mut |el| node.children.push(el.clone()));

        for old_region in node.children_draw_regions.drain(..) {
            ctx.global_content.request_redraw(old_region);
        }

        node.need_relayout = true;
    }
}

impl<T> GetElement for BlockModel<T> {
    fn get_element(&self) -> Element {
        Element::Block(self.node.clone())
    }
}
