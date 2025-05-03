use std::{cell::RefCell, rc::Rc};

use crate::{
    hook::{
        Signal,
        watcher::{WatcherGuard, WatcherList},
    },
    model::{
        EleModel, Model, ModelCreateCtx, VModel,
        component::{Component, ComponentVNode},
    },
    prim_element::{
        Element, EventCallback,
        text::{RenderText, SignalStr, TextStyle},
    },
};

use super::{PrimitiveModel, PrimitiveVnodeWrapper, panic_when_call_unreachable};

#[derive(Default)]
pub struct Text {
    pub text: Option<SignalStr>,
    pub style: Option<Signal<TextStyle>>,
    pub on: Option<EventCallback>,
}

impl Component for Text {
    type ChildProps = ();

    fn create(self, _watcher_list: &mut WatcherList) -> impl ComponentVNode {
        PrimitiveVnodeWrapper(self)
    }
}

impl VModel for PrimitiveVnodeWrapper<Text> {
    type Storage = PrimitiveModel<TextModel>;
    type ParentProps = ();

    fn get_parent_props(&self, _: crate::model::GetParentPropsFn<Self::ParentProps>) {
        panic_when_call_unreachable()
    }

    fn create(&self, ctx: &ModelCreateCtx) -> Self::Storage {
        let model = Rc::new(RefCell::new(TextModel {
            el: Rc::new(RefCell::new(RenderText::new(
                self.0.text.clone(),
                self.0.style.clone(),
                self.0.on.clone(),
                &ctx.el_ctx,
            ))),
        }));

        let mut wl = WatcherList::new();
        wl.watch_borrow_mut(
            &model,
            TextModel::update_text_and_style,
            (self.0.text.clone(), self.0.style.clone()),
        );

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
