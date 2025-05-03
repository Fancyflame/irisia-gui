use irisia_backend::skia_safe::{
    Color, FontMgr,
    textlayout::{
        FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, TextStyle as SkTextStyle,
    },
};

use crate::{hook::Signal, primitive::Point};

use super::{
    Common, EMCreateCtx, EmitEventArgs, EventCallback, RenderTree, Size, SpaceConstraint,
    clip_draw_region, read_or_default,
};

pub type SignalStr = Signal<dyn AsRef<str>>;

pub struct RenderText {
    text: Option<SignalStr>,
    style: Option<Signal<TextStyle>>,
    paragraph: Option<Paragraph>,
    common: Common,
}

#[derive(PartialEq, Clone)]
pub struct TextStyle {
    pub font_size: f32,
    pub font_color: Color,
}

impl TextStyle {
    const DEFAULT: Self = Self {
        font_size: 20.0,
        font_color: Color::BLACK,
    };
}

impl Default for TextStyle {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl RenderText {
    pub fn new(
        text: Option<SignalStr>,
        style: Option<Signal<TextStyle>>,
        event_callback: Option<EventCallback>,
        ctx: &EMCreateCtx,
    ) -> Self {
        Self {
            text,
            style,
            paragraph: None,
            common: Common::new(event_callback, ctx),
        }
    }

    pub fn text_updated(&mut self) {
        self.paragraph = None;
        self.common.cached_layout.take();
        self.common.request_redraw();
        self.common.request_relayout();
    }
}

impl RenderTree for RenderText {
    fn render(&mut self, args: super::RenderArgs, location: Point) {
        let draw_region = self.common.set_rendered(location);
        if !args.needs_redraw(draw_region) {
            return;
        }

        let paragraph = self
            .paragraph
            .get_or_insert_with(|| build_paragraph(&self.text, &self.style));

        args.canvas.save();
        clip_draw_region(args.canvas, draw_region);
        paragraph.paint(args.canvas, location);
        args.canvas.restore();
    }

    fn emit_event(&mut self, args: EmitEventArgs) {
        self.common.use_callback(args);
    }

    fn layout(&mut self, constraint: Size<SpaceConstraint>) -> Size<f32> {
        let layout_fn = |constraint: Size<SpaceConstraint>| {
            let paragraph = self
                .paragraph
                .get_or_insert_with(|| build_paragraph(&self.text, &self.style));

            let w = match constraint.width {
                SpaceConstraint::Exact(width) | SpaceConstraint::Available(width) => width,
                SpaceConstraint::MinContent => paragraph.min_intrinsic_width(),
                SpaceConstraint::MaxContent => paragraph.max_intrinsic_width(),
            };
            paragraph.layout(w);

            Size {
                width: fit_constraint(paragraph.max_width(), constraint.width),
                height: fit_constraint(paragraph.height(), constraint.height),
            }
        };

        self.common.use_cached_layout(constraint, false, layout_fn)
    }

    fn set_callback(&mut self, callback: EventCallback) {
        self.common.event_callback = Some(callback);
    }
}

fn build_paragraph(text: &Option<SignalStr>, style: &Option<Signal<TextStyle>>) -> Paragraph {
    let mut font_collection = FontCollection::new();
    font_collection.set_default_font_manager(FontMgr::new(), None);

    let style = read_or_default(style, &TextStyle::DEFAULT);
    ParagraphBuilder::new(&ParagraphStyle::new(), font_collection)
        .push_style(
            SkTextStyle::new()
                .set_color(style.font_color)
                .set_font_size(style.font_size),
        )
        .add_text(read_or_default(text, &"").as_ref())
        .build()
}

fn fit_constraint(computed: f32, constraint: SpaceConstraint) -> f32 {
    match constraint {
        SpaceConstraint::MinContent | SpaceConstraint::MaxContent => computed,
        SpaceConstraint::Exact(exact) => exact,
        SpaceConstraint::Available(available) => computed.min(available),
    }
}
