use std::{cell::RefCell, rc::Rc};

use crate::{
    hook::{
        reactive::{Reactive, WeakReactive},
        Signal,
    },
    model::{
        component::Component,
        control_flow::{common_vmodel::BoxedModel, CommonVModel},
        EleModel, Model, ModelCreateCtx, VModel, VNode,
    },
    prim_element::{
        block::{LayoutFn, RenderBlock, Tree},
        EMCreateCtx, Element, EventCallback,
    },
    primitive::{Point, Region},
};

use super::{panic_when_call_unreachable, read_or_default, PrimitiveVnodeWrapper};

#[derive(Default)]
pub struct Block {
    pub layout_fn: Option<Signal<LayoutFn>>,
    pub children: Option<Signal<dyn CommonVModel<()>>>,
    pub on: Option<EventCallback>,
}

pub struct BlockModel {
    el: Rc<RefCell<RenderBlock>>,
    children: BoxedModel,
    ctx: ModelCreateCtx,
}

impl Component for Block {
    type Created = ();
    type ChildProps = ();

    fn create(self) -> ((), impl VNode) {
        ((), PrimitiveVnodeWrapper(self))
    }
}

impl VModel for PrimitiveVnodeWrapper<Block> {
    type Storage = Reactive<BlockModel>;

    fn create(&self, ctx: &ModelCreateCtx) -> Self::Storage {
        Reactive::builder()
            .dep(BlockModel::update_layout_fn, self.0.layout_fn.clone())
            .dep(BlockModel::update_children, self.0.children.clone())
            .build_cyclic(|weak| BlockModel::create(weak, &self.0, &ctx.el_ctx))
    }

    fn update(&self, _: &mut Self::Storage, _: &ModelCreateCtx) {
        panic_when_call_unreachable()
    }
}

fn visit_into_vec<T: Model>(model: &T) -> Vec<Element> {
    let mut vec = Vec::new();
    model.visit(&mut |el| vec.push(el));
    vec
}

impl BlockModel {
    pub(crate) fn create(weak: &WeakReactive<Self>, props: &Block, el_ctx: &EMCreateCtx) -> Self {
        let ctx = ModelCreateCtx {
            el_ctx: el_ctx.clone(),
            parent: Some(weak.clone()),
        };

        let children = match &props.children {
            Some(sig) => sig.common_create(&ctx),
            None => CommonVModel::<()>::common_create(&(), &ctx),
        };

        let prim_block = Rc::new(RefCell::new(RenderBlock::new(
            Tree {
                layout_fn: read_or_default(&props.layout_fn, DEFAULT_LAYOUT_FN),
                children: visit_into_vec(&children),
            },
            props.on.clone(),
            el_ctx,
        )));

        BlockModel {
            el: prim_block,
            children,
            ctx,
        }
    }

    fn update_layout_fn(&mut self, layout_fn: Option<&LayoutFn>) {
        self.el.borrow_mut().update_tree().layout_fn = *layout_fn.unwrap();
    }

    fn update_children(&mut self, children: Option<&(dyn CommonVModel<()> + '_)>) {
        let Some(vmodel) = children else {
            return;
        };
        vmodel.update(&mut self.children, &self.ctx);
        self.submit_children();
    }

    pub(crate) fn submit_children(&self) {
        let mut guard = self.el.borrow_mut();
        let mut guard2 = guard.update_tree();
        let dst = &mut guard2.children;
        dst.clear();
        self.children.visit(&mut |el| dst.push(el));
    }
}

impl Model for BlockModel {
    fn visit(&self, f: &mut dyn FnMut(Element)) {
        f(self.get_element())
    }
}

impl EleModel for BlockModel {
    fn get_element(&self) -> Element {
        self.el.clone()
    }
}

pub const DEFAULT_LAYOUT_FN: LayoutFn = default_layout_fn;
fn default_layout_fn(size: Point, elements: &[Element], region_buffer: &mut Vec<Region>) {
    region_buffer.resize(
        elements.len(),
        Region {
            left_top: (0.0, 0.0).into(),
            right_bottom: size,
        },
    );
}
