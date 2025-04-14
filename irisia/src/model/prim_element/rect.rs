use std::{cell::RefCell, rc::Rc};

use crate::{
    hook::{reactive::Reactive, Signal},
    model::{component::Component, Model, ModelCreateCtx, VModel},
    prim_element::{
        rect::{RectStyle, RenderRect},
        EventCallback, GetElement,
    },
};

use super::{read_or_default, PrimitiveVModelWrapper};

#[derive(Default)]
pub struct Rect {
    pub style: Option<Signal<RectStyle>>,
    pub on: Option<EventCallback>,
}

impl Component for Rect {
    type Created = ();
    fn create(self) -> (Self::Created, impl crate::model::VModel) {
        ((), PrimitiveVModelWrapper(self))
    }
}

impl VModel for PrimitiveVModelWrapper<Rect> {
    type Storage = Reactive<RectModel>;

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
        unreachable!("primitive v-model never updates");
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

impl Model for RectModel {
    fn visit(&self, f: &mut dyn FnMut(crate::prim_element::Element)) {
        f(self.el.get_element())
    }
}
