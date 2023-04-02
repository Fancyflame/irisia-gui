use cream_core::{
    element::{Element, Frame, NeverInitalized},
    skia_safe::{
        font_style::Width,
        textlayout::{FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, TextStyle},
        Color, FontMgr, FontStyle, Point,
    },
    structure::VisitIter,
    style::{StyleColor, StyleContainer},
    StyleReader,
};
use styles::*;

pub mod styles;

pub struct TextBox {
    font_collection: FontCollection,
    paragraph: Option<Paragraph>,
}

#[derive(StyleReader)]
struct TextBoxStyles {
    size: StyleFontSize,
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
        }: cream_core::element::Frame<
            Self,
            impl StyleContainer,
            impl VisitIter<Self::ChildProps<'a>>,
        >,
    ) -> cream_core::Result<()> {
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
                .set_font_size(style.size.0.to_physical() as _);

            text_style.set_color(match style.color {
                Some(c) => c.0,
                None => Color::BLACK,
            });

            text_style
        };

        paragraph_builder
            .push_style(&text_style)
            .add_text(props.text);

        let mut paragraph = paragraph_builder.build();
        paragraph.layout((drawing_region.1 .0 - drawing_region.0 .0) as _);
        paragraph.paint(
            content.canvas(),
            Point::new(drawing_region.0 .0 as _, drawing_region.0 .1 as _),
        );
        self.paragraph = Some(paragraph);

        Ok(())
    }
}
