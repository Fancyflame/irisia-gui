use std::{cell::RefCell, rc::Rc};

use irisia_backend::skia_safe::{
    textlayout::{FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, TextStyle},
    Color, FontMgr,
};

use crate::{model2::VModel, primitive::Region};

use super::{Element, GetElement, Handle, RenderTree};

pub struct RenderText {
    prop: Text,
    paragraph: Option<Paragraph>,
    prev_draw_region: Option<Region>,
}

impl RenderTree for RenderText {
    fn render(&mut self, args: super::RenderArgs, draw_region: crate::primitive::Region) {
        if !args.needs_redraw(draw_region) {
            return;
        }

        self.prev_draw_region = Some(draw_region);
        let para = self
            .paragraph
            .get_or_insert_with(|| self.prop.build_paragraph());
        para.layout(draw_region.right_bottom.0 - draw_region.left_top.0);
        para.paint(args.canvas, draw_region.left_top);
    }
}

#[derive(PartialEq, Default, Clone)]
pub struct Text {
    pub text: String,
    pub font_size: f32,
    pub font_color: Color,
}

pub struct TextModel {
    node: Handle<RenderText>,
}

impl VModel for Text {
    type Storage = TextModel;
    fn create(&self, _: &crate::el_model::EMCreateCtx) -> Self::Storage {
        TextModel {
            node: Rc::new(RefCell::new(RenderText {
                prop: self.clone(),
                paragraph: None,
                prev_draw_region: None,
            })),
        }
    }
    fn update(&self, storage: &mut Self::Storage, ctx: &crate::el_model::EMCreateCtx) {
        let mut node = storage.node.borrow_mut();
        if *self != node.prop {
            node.paragraph = None;
            if let Some(dr) = node.prev_draw_region.take() {
                ctx.global_content.request_redraw(dr);
            }
        }
    }
}

impl Text {
    fn build_paragraph(&self) -> Paragraph {
        let mut font_collection = FontCollection::new();
        font_collection.set_default_font_manager(FontMgr::new(), None);

        ParagraphBuilder::new(&ParagraphStyle::new(), font_collection)
            .push_style(
                TextStyle::new()
                    .set_color(self.font_color)
                    .set_font_size(self.font_size),
            )
            .add_text(&self.text)
            .build()
    }
}

impl GetElement for TextModel {
    fn get_element(&self) -> super::Element {
        Element::Text(self.node.clone())
    }
}
