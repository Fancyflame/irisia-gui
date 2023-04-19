use std::sync::{Arc, Mutex};

use irisia::skia_safe::{Color4f, ColorSpace, Paint};
use irisia_core::{
    element::{Element, Frame, NeverInitalized, RuntimeInit},
    primary::Point,
    skia_safe::{
        font_style::Width,
        textlayout::{FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, TextStyle},
        Color, FontMgr, FontStyle, Point as SkiaPoint,
    },
    structure::VisitIter,
    style::{StyleColor, StyleContainer},
    StyleReader,
};
use styles::*;

mod selection;
pub mod styles;

type SelectionRange = Arc<Mutex<Option<(Point, Point)>>>;

pub struct TextBox {
    font_collection: FontCollection,
    paragraph: Option<Paragraph>,
    selection_range: SelectionRange,
}

#[derive(StyleReader)]
struct TextBoxStyles {
    font_size: StyleFontSize,
    slant: StyleFontSlant,
    weight: StyleFontWeight,
    color: Option<StyleColor>,
}

impl Default for TextBox {
    fn default() -> Self {
        let mut font_collection = FontCollection::new();
        font_collection.set_default_font_manager(FontMgr::new(), None);
        TextBox {
            font_collection,
            paragraph: None,
            selection_range: Default::default(),
        }
    }
}

#[derive(Default)]
pub struct Props<'a> {
    pub text: &'a str,
}

impl Element for TextBox {
    type Props<'a> = Props<'a>;
    type ChildProps<'a> = NeverInitalized;

    fn render<'a>(
        &mut self,
        Frame {
            props,
            styles,
            drawing_region,
            mut content,
            ..
        }: irisia_core::element::Frame<
            Self,
            impl StyleContainer,
            impl VisitIter<Self::ChildProps<'a>>,
        >,
    ) -> irisia_core::Result<()> {
        let style = TextBoxStyles::read_style(styles);

        let para_style = {
            let mut ps = ParagraphStyle::new();
            ps.set_height((drawing_region.1 .1 - drawing_region.0 .1) as _);
            ps
        };

        let mut paragraph_builder = ParagraphBuilder::new(&para_style, &self.font_collection);

        let text_style = {
            let mut text_style = TextStyle::new();
            text_style
                .set_font_style(FontStyle::new(style.weight.0, Width::NORMAL, style.slant.0))
                .set_font_size(style.font_size.0.to_physical() as _);

            text_style.set_color(match style.color {
                Some(c) => c.0,
                None => Color::BLACK,
            });
            text_style
        };

        match self.get_selection_range(props.text) {
            Some(range) => {
                let (start, end) = (range.start, range.end);
                let mut selection_style = text_style.clone();
                to_selection_style(&mut selection_style);

                paragraph_builder
                    .push_style(&text_style)
                    .add_text(&props.text[..start])
                    .push_style(&selection_style)
                    .add_text(&props.text[range])
                    .push_style(&text_style)
                    .add_text(&props.text[end..]);
            }
            None => {
                paragraph_builder
                    .push_style(&text_style)
                    .add_text(props.text);
            }
        }

        let mut paragraph = paragraph_builder.build();
        paragraph.layout((drawing_region.1 .0 - drawing_region.0 .0) as _);
        paragraph.paint(
            content.canvas(),
            SkiaPoint::new(drawing_region.0 .0 as _, drawing_region.0 .1 as _),
        );
        self.paragraph = Some(paragraph);

        Ok(())
    }

    fn start_runtime(init: RuntimeInit<Self>) {
        init.element_handle
            .clone()
            .spawn(Self::start_selection_runtime(
                init.window_event_dispatcher,
                init.element_handle,
                init.app.blocking_lock().selection_range.clone(),
            ));
    }
}

fn to_selection_style(style: &mut TextStyle) {
    style.set_color(Color::WHITE);
    style.set_background_color(&Paint::new(
        Color4f::from(Color::from_argb(0xee, 0x4d, 0x90, 0xfe)),
        &ColorSpace::new_srgb(),
    ));
}
