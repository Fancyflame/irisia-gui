use std::{cell::RefCell, rc::Rc};

use crate::{
    WeakHandle,
    hook::{
        Signal,
        watcher::{WatcherGuard, WatcherList},
    },
    model::{
        EleModel, Model, ModelCreateCtx, VModel, VNode,
        component::Component,
        control_flow::{
            CommonVModel,
            common_vmodel::{BoxedModel, DynVModel},
        },
    },
    prim_element::{
        EMCreateCtx, Element, EventCallback,
        block::{BlockLayout, BlockStyle, ElementList, InitRenderBlock, RenderBlock},
    },
};

use super::{PrimitiveModel, PrimitiveVnodeWrapper, panic_when_call_unreachable};

pub struct Block<Cd> {
    pub display: Option<Signal<dyn BlockLayout<Cd>>>,
    pub style: Option<Signal<BlockStyle>>,
    pub children: Option<Signal<DynVModel<Cd>>>,
    pub on: Option<EventCallback>,
}

impl<Cd> Default for Block<Cd> {
    fn default() -> Self {
        Self {
            display: None,
            style: None,
            children: None,
            on: None,
        }
    }
}

pub struct BlockModel<Cd> {
    el: Rc<RefCell<RenderBlock<Cd>>>,
    children: BoxedModel<Cd>,
    ctx: ModelCreateCtx,
}

impl<Cd: 'static> Component for Block<Cd> {
    fn create(self, _watcher_list: &mut WatcherList) -> impl VNode<()> + use<Cd> {
        PrimitiveVnodeWrapper(self)
    }
}

impl<Cd: 'static> VModel<()> for PrimitiveVnodeWrapper<Block<Cd>> {
    type Storage = PrimitiveModel<BlockModel<Cd>>;

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
        .watch_borrow_mut(
            &model,
            BlockModel::<Cd>::update_children,
            self.0.children.clone(),
        );

        PrimitiveModel {
            model,
            _watcher_list: wl,
        }
    }

    fn update(&self, _: &mut Self::Storage, _: &ModelCreateCtx) {
        panic_when_call_unreachable()
    }
}

fn visit_into_list<T, Cd>(model: &T) -> ElementList<Cd>
where
    T: Model<Cd>,
{
    let mut vec = ElementList::new();
    model.visit(&mut |el, cd| vec.push(el, cd));
    vec
}

impl<Cd: 'static> BlockModel<Cd> {
    pub(crate) fn create(
        self_weak: &WeakHandle<Self>,
        props: &Block<Cd>,
        el_ctx: &EMCreateCtx,
    ) -> Self {
        let mut children_ctx = None;

        let prim_block = Rc::new_cyclic(|render_block_weak| {
            let ctx = ModelCreateCtx {
                el_ctx: EMCreateCtx {
                    global_content: el_ctx.global_content.clone(),
                    parent: Some(render_block_weak.clone() as _),
                },
                parent: Some(self_weak.clone() as _),
            };

            let children = match &props.children {
                Some(sig) => sig.common_create(&ctx),
                None => ().common_create(&ctx),
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

    fn update_children(&mut self, children: Option<&DynVModel<Cd>>) {
        let Some(vmodel) = children else {
            return;
        };
        vmodel.update(&mut self.children, &self.ctx);
        self.submit_children();
    }
}

impl<Cd: 'static> Model<()> for BlockModel<Cd> {
    fn visit(&self, f: &mut dyn FnMut(Element, ())) {
        f(self.el.clone(), ())
    }
}

impl<Cd: 'static> EleModel<()> for BlockModel<Cd> {
    fn get_element(&self) -> (Element, ()) {
        (self.el.clone(), ())
    }
}

pub(crate) trait SubmitChildren {
    fn submit_children(&self);
}

impl<Cd: 'static> SubmitChildren for BlockModel<Cd> {
    fn submit_children(&self) {
        let mut guard = self.el.borrow_mut();
        let mut guard2 = guard.update_children();
        self.children.visit(&mut |el, cd| guard2.push(el, cd));
    }
}
