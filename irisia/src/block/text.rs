use irisia_backend::skia_safe::{
    textlayout::{FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, TextStyle},
    Color, FontMgr,
};

use crate::model2::VModel;

use super::RenderUnit;

#[derive(PartialEq)]
pub struct Text {
    pub text: String,
    pub font_size: f32,
    pub font_color: Color,
}

pub struct TextCache {
    prop: Text,
    paragraph: Option<Paragraph>,
}

impl VModel for Text {
    type Storage = TextCache;
    fn create(self, _: &crate::el_model::EMCreateCtx) -> Self::Storage {
        TextCache {
            prop: self,
            paragraph: None,
        }
    }
    fn update(self, storage: &mut Self::Storage, _: &crate::el_model::EMCreateCtx) {
        if self != storage.prop {
            storage.paragraph = None;
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

impl RenderUnit for TextCache {
    fn render(&mut self, args: super::RenderArgs, draw_region: crate::primitive::Region) {
        if !args.needs_redraw(draw_region) {
            return;
        }

        let para = self
            .paragraph
            .get_or_insert_with(|| self.prop.build_paragraph());
        para.layout(draw_region.right_bottom.0 - draw_region.left_top.0);
        para.paint(args.canvas, draw_region.left_top);
    }
}
