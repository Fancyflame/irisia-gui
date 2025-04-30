use std::{cell::RefCell, rc::Rc};

use crate::{
    hook::{
        reactive::{Reactive, WeakReactive},
        Signal,
    },
    model::{
        component::Component,
        control_flow::{
            common_vmodel::{BoxedModel, DynVModel},
            miscellaneous::Empty,
            CommonVModel,
        },
        EleModel, Model, ModelCreateCtx, VModel, VNode,
    },
    prim_element::{
        block::{
            layout::LayoutChildren, BlockLayout, BlockStyle, ElementList, InitRenderBlock,
            LayoutFn, RenderBlock,
        },
        EMCreateCtx, Element, EventCallback, Size, SpaceConstraint,
    },
    primitive::{Point, Region},
};

use super::{panic_when_call_unreachable, read_or_default, PrimitiveVnodeWrapper};

#[derive(Default)]
pub struct Block {
    pub display: Option<Signal<dyn BlockLayout>>,
    pub style: Option<Signal<BlockStyle>>,
    pub children: Option<Signal<DynVModel<()>>>,
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

    fn create(self) -> ((), impl VNode<ParentProps = ()>) {
        ((), PrimitiveVnodeWrapper(self))
    }
}

impl VModel for PrimitiveVnodeWrapper<Block> {
    type Storage = Reactive<BlockModel>;
    type ParentProps = ();

    fn get_parent_props(&self, _: crate::model::GetParentPropsFn<Self::ParentProps>) {
        panic_when_call_unreachable()
    }

    fn create(&self, ctx: &ModelCreateCtx) -> Self::Storage {
        Reactive::builder()
            .dep(BlockModel::update_layouter, self.0.display.clone())
            .dep(BlockModel::update_children, self.0.children.clone())
            .build_cyclic(|weak| BlockModel::create(weak, &self.0, &ctx.el_ctx))
    }

    fn update(&self, _: &mut Self::Storage, _: &ModelCreateCtx) {
        panic_when_call_unreachable()
    }
}

fn visit_into_list<T: Model>(model: &T) -> ElementList {
    let mut vec = ElementList::new();
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
            None => Empty::<()>::new().common_create(&ctx),
        };

        let prim_block = Rc::new(RefCell::new(RenderBlock::new(InitRenderBlock {
            style: props.style.clone(),
            children: visit_into_list(&children),
            layouter: props.display.clone(),
            event_callback: props.on.clone(),
            ctx: el_ctx,
        })));

        BlockModel {
            el: prim_block,
            children,
            ctx,
        }
    }

    fn update_layouter(&mut self) {
        self.el.borrow_mut().layouter_updated();
    }

    fn update_children(&mut self, children: Option<&DynVModel<()>>) {
        let Some(vmodel) = children else {
            return;
        };
        vmodel.update(&mut self.children, &self.ctx);
        self.submit_children();
    }

    pub(crate) fn submit_children(&self) {
        let mut guard = self.el.borrow_mut();
        let mut guard2 = guard.update_children();
        self.children.visit(&mut |el| guard2.push(el));
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
