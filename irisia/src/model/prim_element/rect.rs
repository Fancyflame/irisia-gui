use std::{cell::RefCell, rc::Rc};

use crate::{
    hook::{reactive::Reactive, Signal},
    model::{component::Component, Model, VModel},
    prim_element::{
        rect::{RectStyle, RenderRect},
        GetElement,
    },
};

use super::{read_or_default, PrimitiveVModelWrapper};

#[derive(Default)]
pub struct Rect {
    pub style: Option<Signal<RectStyle>>,
}

impl Component for Rect {
    type Created = ();
    fn create(self) -> (Self::Created, impl crate::model::VModel) {
        ((), PrimitiveVModelWrapper(self))
    }
}

impl VModel for PrimitiveVModelWrapper<Rect> {
    type Storage = Reactive<RectModel>;

    fn create(&self, ctx: &crate::prim_element::EMCreateCtx) -> Self::Storage {
        let init_style = read_or_default(&self.0.style, RectStyle::default());
        let init_state = RectModel {
            el: Rc::new(RefCell::new(RenderRect::new(
                init_style,
                Box::new(|_| {}),
                ctx,
            ))),
        };

        Reactive::builder(init_state)
            .dep(RectModel::update_style, self.0.style.clone())
            .build()
    }

    fn update(&self, _: &mut Self::Storage, _: &crate::prim_element::EMCreateCtx) {
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
