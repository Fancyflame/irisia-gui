use std::{cell::RefCell, rc::Rc};

use crate::{
    hook::{reactive::Reactive, Signal},
    model::{
        component::Component,
        control_flow::{common_vmodel::BoxedModel, CommonVModel},
        Model, VModel,
    },
    prim_element::{
        block::{LayoutFn, RenderBlock, Tree},
        EMCreateCtx, Element, GetElement,
    },
    primitive::{Point, Region},
};

use super::PrimitiveVModelWrapper;

pub struct BlockModel {
    el: Rc<RefCell<RenderBlock>>,
    children: BoxedModel,
    ctx: EMCreateCtx,
}

#[derive(Default)]
pub struct Block {
    pub layout_fn: Option<Signal<LayoutFn>>,
    pub children: Option<Signal<dyn CommonVModel>>,
}

impl Component for Block {
    type Created = ();
    fn create(self) -> ((), impl VModel) {
        ((), PrimitiveVModelWrapper(self))
    }
}

impl VModel for PrimitiveVModelWrapper<Block> {
    type Storage = Reactive<BlockModel>;

    fn create(&self, ctx: &crate::prim_element::EMCreateCtx) -> Self::Storage {
        let prim_block = Rc::new(RefCell::new(RenderBlock::new(
            Tree {
                layout_fn: default_layout_fn,
                children: vec![],
            },
            Box::new(|_| {}),
            &ctx,
        )));

        let init_state = BlockModel {
            el: prim_block,
            children: ().common_create(&ctx),
            ctx: ctx.clone(),
        };

        Reactive::builder(init_state)
            .dep_call(BlockModel::update_layout_fn, self.0.layout_fn.clone(), true)
            .dep_call(BlockModel::update_children, self.0.children.clone(), true)
            .build()
    }

    fn update(&self, _: &mut Self::Storage, _: &crate::prim_element::EMCreateCtx) {
        unreachable!("primitive v-model never updates");
    }
}

pub fn default_layout_fn(size: Point, elements: &[Element], region_buffer: &mut Vec<Region>) {
    region_buffer.resize(
        elements.len(),
        Region {
            left_top: (0.0, 0.0).into(),
            right_bottom: size,
        },
    );
}

impl Model for BlockModel {
    fn visit(&self, f: &mut dyn FnMut(Element)) {
        f(self.el.get_element())
    }
}

impl BlockModel {
    fn update_layout_fn(&mut self, layout_fn: Option<&LayoutFn>) {
        if let Some(layout_fn) = layout_fn {
            self.el.borrow_mut().update_tree().layout_fn = *layout_fn;
        }
    }

    fn update_children(&mut self, children: Option<&(dyn CommonVModel + '_)>) {
        let Some(vmodel) = children else {
            return;
        };
        vmodel.update(&mut self.children, &self.ctx);

        let mut guard = self.el.borrow_mut();
        let mut guard2 = guard.update_tree();
        let dst = &mut guard2.children;
        dst.clear();
        self.children.visit(&mut |el| dst.push(el));
    }
}
