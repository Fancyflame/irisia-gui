use std::{cell::RefCell, rc::Rc};

use crate::{
    WeakHandle,
    hook::{
        Signal,
        watcher::{WatcherGuard, WatcherList},
    },
    model::{
        EleModel, Model, ModelCreateCtx, VModel,
        component::{Component, ComponentVNode},
        control_flow::{
            CommonVModel,
            common_vmodel::{BoxedModel, DynVModel},
            miscellaneous::Empty,
        },
    },
    prim_element::{
        EMCreateCtx, Element, EventCallback,
        block::{BlockLayout, BlockStyle, ElementList, InitRenderBlock, RenderBlock},
    },
};

use super::{PrimitiveModel, PrimitiveVnodeWrapper, panic_when_call_unreachable};

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
    type ChildProps = ();

    fn create(self, _watcher_list: &mut WatcherList) -> impl ComponentVNode {
        PrimitiveVnodeWrapper(self)
    }
}

impl VModel for PrimitiveVnodeWrapper<Block> {
    type Storage = PrimitiveModel<BlockModel>;
    type ParentProps = ();

    fn get_parent_props(&self, _: crate::model::GetParentPropsFn<Self::ParentProps>) {
        panic_when_call_unreachable()
    }

    fn create(&self, ctx: &ModelCreateCtx) -> Self::Storage {
        let mut wl = WatcherList::new();
        let model =
            Rc::new_cyclic(|weak| RefCell::new(BlockModel::create(weak, &self.0, &ctx.el_ctx)));

        wl.watch_borrow_mut(
            &model,
            |this, _| this.layouter_updated(),
            self.0.display.clone(),
        )
        .watch_borrow_mut(&model, |this, _| this.style_updated(), self.0.style.clone())
        .watch_borrow_mut(&model, BlockModel::update_children, self.0.children.clone());

        PrimitiveModel {
            model,
            _watcher_list: wl,
        }
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
    pub(crate) fn create(
        self_weak: &WeakHandle<Self>,
        props: &Block,
        el_ctx: &EMCreateCtx,
    ) -> Self {
        let mut children_ctx = None;

        let prim_block = Rc::new_cyclic(|render_block_weak| {
            let ctx = ModelCreateCtx {
                el_ctx: EMCreateCtx {
                    global_content: el_ctx.global_content.clone(),
                    parent: Some(render_block_weak.clone()),
                },
                parent: Some(self_weak.clone() as _),
            };

            let children = match &props.children {
                Some(sig) => sig.common_create(&ctx),
                None => Empty::<()>::new().common_create(&ctx),
            };

            let ret = RefCell::new(RenderBlock::new(InitRenderBlock {
                this: render_block_weak.clone() as _,
                style: props.style.clone(),
                children: visit_into_list(&children),
                layouter: props.display.clone(),
                event_callback: props.on.clone(),
                ctx: el_ctx,
            }));

            children_ctx = Some((children, ctx));
            ret
        });

        let Some((children, ctx)) = children_ctx else {
            unreachable!();
        };

        BlockModel {
            el: prim_block,
            children,
            ctx,
        }
    }

    fn style_updated(&self) {
        self.el.borrow_mut().style_updated();
    }

    fn layouter_updated(&self) {
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
