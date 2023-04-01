use cream_core::{
    element::{Element, NeverInitalized},
    read_style,
    skia_safe::{
        font_style::Width,
        textlayout::{FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, TextStyle},
        Color, FontMgr, FontStyle, Point,
    },
    structure::VisitIter,
    style::StyleColor,
};
use styles::*;

pub mod styles;

pub struct TextBox {
    font_collection: FontCollection,
    paragraph: Option<Paragraph>,
}

#[derive(Default)]
pub struct Props<'a> {
    pub text: &'a str,
}

impl Element for TextBox {
    type Props<'a> = Props<'a>;
    type ChildProps<'a> = NeverInitalized;

    fn create() -> Self {
        let mut font_collection = FontCollection::new();
        font_collection.set_default_font_manager(FontMgr::new(), None);
        TextBox {
            font_collection,
            paragraph: None,
        }
    }

    fn render<'r>(
        &mut self,
        props: Self::Props<'_>,
        styles: &impl cream_core::style::StyleContainer,
        drawing_region: cream_core::primary::Region,
        _cache_box_for_children: &mut cream_core::CacheBox,
        _event_dispatcher: &cream_core::event::EventDispatcher,
        _children: cream_core::structure::Slot<
            impl cream_core::structure::StructureBuilder + VisitIter<Self::ChildProps<'r>>,
        >,
        mut content: cream_core::element::RenderContent,
    ) -> cream_core::Result<()> {
        read_style!(styles => {
            size: StyleFontSize,
            slant: StyleFontSlant,
            weight: StyleFontWeight,
            color: Option<StyleColor>,
        });

        let para_style = {
            let mut ps = ParagraphStyle::new();
            ps.set_height((drawing_region.1 .1 - drawing_region.0 .1) as _);
            ps
        };

        let mut paragraph_builder = ParagraphBuilder::new(&para_style, &self.font_collection);

        let text_style = {
            let mut text_style = TextStyle::new();
            text_style
                .set_font_style(FontStyle::new(weight.0, Width::NORMAL, slant.0))
                .set_font_size(size.0.to_physical() as _);

            text_style.set_color(match color {
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
