use irisia_backend::skia_safe::{
    Color, FontMgr,
    textlayout::{
        FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, TextStyle as SkTextStyle,
    },
};

use crate::{hook::Signal, primitive::Point};

use super::{
    Common, EMCreateCtx, EventCallback, RenderTree, Size, WeakElement,
    layout::{LayoutInput, SpaceConstraint},
    read_or_default,
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
        this: WeakElement,
        text: Option<SignalStr>,
        style: Option<Signal<TextStyle>>,
        event_callback: Option<EventCallback>,
        ctx: &EMCreateCtx,
    ) -> Self {
        Self {
            text,
            style,
            paragraph: None,
            common: Common::new(this, event_callback, ctx),
        }
    }

    pub fn text_updated(&mut self) {
        self.paragraph = None;
        self.common.request_repaint();
        self.common.request_reflow();
    }
}

impl RenderTree for RenderText {
    fn render(&mut self, args: super::RenderArgs, draw_location: Point<f32>) {
        let paragraph = self
            .paragraph
            .get_or_insert_with(|| build_paragraph(&self.text, &self.style));

        paragraph.paint(args.canvas, draw_location);
    }

    fn compute_layout(
        &mut self,
        LayoutInput {
            constraint,
            length_standard: _,
        }: LayoutInput,
    ) -> Size<f32> {
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
    }

    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }

    fn children_emit_event(&mut self, _: &mut super::EmitEventArgs) {}
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
