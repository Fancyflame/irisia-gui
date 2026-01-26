use crate::{
    self as irisia,
    hook::watcher::Watcher,
    model::{UnitVModel, VisitModelFn},
};
use irisia_macros::Property;
use std::{cell::RefCell, rc::Rc};

use crate::{
    hook::Signal,
    model::{Model, ModelCreateCtx, UnitModel, VModel, component::Component},
    prim_element::{
        Element, EventCallback,
        text::{RenderText, SignalStr, TextStyle},
    },
};

use super::{PrimitiveModel, PrimitiveVnodeWrapper, panic_when_call_unreachable};

#[derive(Default, Property)]
pub struct Text {
    pub text: Option<SignalStr>,
    pub style: Option<Signal<TextStyle>>,
    pub on: Option<EventCallback>,
}

impl Component for Text {
    fn create(self, _watcher_list: &mut Vec<Watcher>) -> impl UnitVModel + use<> {
        PrimitiveVnodeWrapper(self)
    }
}

impl VModel for PrimitiveVnodeWrapper<Text> {
    type Storage = PrimitiveModel<TextModel>;

    fn create(&self, ctx: &ModelCreateCtx) -> Self::Storage {
        let model = Rc::new(RefCell::new(TextModel {
            el: Rc::new_cyclic(|weak| {
                RefCell::new(RenderText::new(
                    weak.clone() as _,
                    self.0.text.clone(),
                    self.0.style.clone(),
                    self.0.on.clone(),
                    &ctx.el_ctx,
                ))
            }),
        }));

        let wl = vec![Watcher::with(
            &model,
            (self.0.text.clone(), self.0.style.clone()),
            TextModel::update_text_and_style,
        )];

        PrimitiveModel {
            _watcher_list: wl,
            model,
        }
    }

    fn update(&self, _: &mut Self::Storage, _: &ModelCreateCtx) {
        panic_when_call_unreachable()
    }
}

pub struct TextModel {
    el: Rc<RefCell<RenderText>>,
}

impl TextModel {
    fn update_text_and_style(
        &mut self,
        inputs: (Option<&(dyn AsRef<str> + 'static)>, Option<&TextStyle>),
    ) {
        if let (None, None) = inputs {
            return;
        }

        self.el.borrow_mut().text_updated();
    }
}

impl Model for TextModel {
    fn visit_raw(&self, f: VisitModelFn) {
        f(self.el.clone(), None)
    }
}
