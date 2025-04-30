use std::{cell::RefCell, rc::Rc};

use crate::{
    hook::{reactive::Reactive, Signal},
    model::{component::Component, EleModel, Model, ModelCreateCtx, VModel, VNode},
    prim_element::{
        text::{RenderText, SignalStr, TextStyle},
        Element, EventCallback,
    },
};

use super::{panic_when_call_unreachable, read_or_default, PrimitiveVnodeWrapper};

#[derive(Default)]
pub struct Text {
    pub text: Option<SignalStr>,
    pub style: Option<Signal<TextStyle>>,
    pub on: Option<EventCallback>,
}

impl Component for Text {
    type Created = ();
    type ChildProps = ();

    fn create(self) -> ((), impl VNode<ParentProps = ()>) {
        ((), PrimitiveVnodeWrapper(self))
    }
}

impl VModel for PrimitiveVnodeWrapper<Text> {
    type Storage = Reactive<TextModel>;
    type ParentProps = ();

    fn get_parent_props(&self, _: crate::model::GetParentPropsFn<Self::ParentProps>) {
        panic_when_call_unreachable()
    }

    fn create(&self, ctx: &ModelCreateCtx) -> Self::Storage {
        let init_state = TextModel {
            el: Rc::new(RefCell::new(RenderText::new(
                self.0.text.clone(),
                self.0.style.clone(),
                self.0.on.clone(),
                &ctx.el_ctx,
            ))),
        };

        Reactive::builder()
            .dep(
                TextModel::update_text_and_style,
                (self.0.text.clone(), self.0.style.clone()),
            )
            .build(init_state)
    }

    fn update(&self, _: &mut Self::Storage, _: &ModelCreateCtx) {
        panic_when_call_unreachable()
    }
}

pub struct TextModel {
    el: Rc<RefCell<RenderText>>,
}

impl TextModel {
    fn update_text_and_style(&mut self, inputs: (Option<&String>, Option<&TextStyle>)) {
        if let (None, None) = inputs {
            return;
        }

        self.el.borrow_mut().text_updated();
    }
}

impl EleModel for TextModel {
    fn get_element(&self) -> Element {
        self.el.clone()
    }
}

impl Model for TextModel {
    fn visit(&self, f: &mut dyn FnMut(Element)) {
        f(self.get_element())
    }
}
