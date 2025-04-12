use irisia_backend::skia_safe::{
    textlayout::{
        FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, TextStyle as SkTextStyle,
    },
    Color, FontMgr,
};

use crate::{application::event2::pointer_event::PointerStateDelta, primitive::Region};

use super::{redraw_guard::RedrawGuard, Common, EMCreateCtx, EventCallback, RenderTree};

pub struct RenderText {
    text: TextSettings,
    paragraph: Option<Paragraph>,
    common: Common,
}

#[derive(PartialEq, Default, Clone)]
pub struct TextSettings {
    pub text: String,
    pub style: TextStyle,
}

#[derive(PartialEq, Default, Clone)]
pub struct TextStyle {
    pub font_size: f32,
    pub font_color: Color,
}

impl RenderText {
    pub fn new(text: TextSettings, event_callback: EventCallback, ctx: &EMCreateCtx) -> Self {
        Self {
            text,
            paragraph: None,
            common: Common::new(event_callback, ctx),
        }
    }

    pub fn update_text(&mut self) -> RedrawGuard<TextSettings> {
        self.paragraph = None;
        RedrawGuard::new(&mut self.text, &mut self.common)
    }
}

impl RenderTree for RenderText {
    fn render(&mut self, args: super::RenderArgs, draw_region: crate::primitive::Region) {
        if !args.needs_redraw(draw_region) {
            return;
        }

        self.common.prev_draw_region = Some(draw_region);
        let para = self
            .paragraph
            .get_or_insert_with(|| self.text.build_paragraph());
        para.layout(draw_region.right_bottom.0 - draw_region.left_top.0);
        para.paint(args.canvas, draw_region.left_top);
    }

    fn emit_event(&mut self, delta: &mut PointerStateDelta, draw_region: Region) {
        self.common.use_callback(delta, draw_region);
    }

    fn set_callback(&mut self, callback: EventCallback) {
        self.common.event_callback = callback;
    }
}

impl TextSettings {
    fn build_paragraph(&self) -> Paragraph {
        let mut font_collection = FontCollection::new();
        font_collection.set_default_font_manager(FontMgr::new(), None);

        ParagraphBuilder::new(&ParagraphStyle::new(), font_collection)
            .push_style(
                SkTextStyle::new()
                    .set_color(self.style.font_color)
                    .set_font_size(self.style.font_size),
            )
            .add_text(&self.text)
            .build()
    }
}
