use std::{cell::RefCell, rc::Rc};

use crate::{
    hook::{reactive::Reactive, Signal},
    model::{component::Component, EleModel, Model, ModelCreateCtx, VModel, VNode},
    prim_element::{
        rect::{RectStyle, RenderRect},
        Element, EventCallback,
    },
};

use super::{panic_when_call_unreachable, read_or_default, PrimitiveVnodeWrapper};

#[derive(Default)]
pub struct Rect {
    pub style: Option<Signal<RectStyle>>,
    pub on: Option<EventCallback>,
}

impl Component for Rect {
    type Created = ();
    type ChildProps = ();

    fn create(self) -> (Self::Created, impl VNode<ParentProps = ()>) {
        ((), PrimitiveVnodeWrapper(self))
    }
}

impl VModel for PrimitiveVnodeWrapper<Rect> {
    type Storage = Reactive<RectModel>;
    type ParentProps = ();

    fn get_parent_props(&self, _: crate::model::GetParentPropsFn<Self::ParentProps>) {
        panic_when_call_unreachable()
    }

    fn create(&self, ctx: &ModelCreateCtx) -> Self::Storage {
        let init_style = read_or_default(&self.0.style, RectStyle::default());

        let init_state = RectModel {
            el: Rc::new(RefCell::new(RenderRect::new(
                init_style,
                self.0.on.clone(),
                &ctx.el_ctx,
            ))),
        };

        Reactive::builder()
            .dep(RectModel::update_style, self.0.style.clone())
            .build(init_state)
    }

    fn update(&self, _: &mut Self::Storage, _: &ModelCreateCtx) {
        panic_when_call_unreachable()
    }
}

pub struct RectModel {
    el: Rc<RefCell<RenderRect>>,
}

impl RectModel {
    fn update_style(&mut self, style: Option<&RectStyle>) {
        *self.el.borrow_mut().update_rect() = style.unwrap().clone();
    }
}

impl EleModel for RectModel {
    fn get_element(&self) -> Element {
        self.el.clone()
    }
}

impl Model for RectModel {
    fn visit(&self, f: &mut dyn FnMut(Element)) {
        f(self.get_element())
    }
}
