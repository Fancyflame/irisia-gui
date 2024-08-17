use std::ops::Range;

use irisia::{
    data_flow::ReadWire, element::ElementInterfaces, skia_safe::{
        font_style::Width,
        textlayout::{FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, TextStyle},
        Color, FontMgr, FontStyle, Point as SkiaPoint,
    }, ReadStyle, UserProps, WriteStyle
};
use irisia::{
    primitive::Region,
    skia_safe::{Color4f, ColorSpace, Paint},
};
use styles::*;

//use self::selection::SelectionRtMgr;

//mod selection;
pub mod styles;

pub struct TextBox {
    props: OwnedProps,
    font_collection: FontCollection,
    paragraph: Option<Paragraph>,
    selection: Option<Range<usize>>,
    //selection_rt_mgr: SelectionRtMgr,
}

#[derive(WriteStyle, PartialEq)]
struct TextBoxStyles {
    font_size: FontSize,
    slant: FontSlant,
    weight: FontWeight,
    color: FontColor,
}

#[derive(UserProps)]
pub struct OwnedProps {
    #[props(required)]
    text: ReadWire<String>,

    #[props(default = false)]
    user_select: ReadWire<bool>,
}

impl ElementInterfaces for TextBox {
    type BlankProps = TextBoxProps;

    fn render(
        &mut self,
        this: &RcElementModel<Self>,
        mut content: RenderElement,
    ) -> irisia::Result<()> {
        let draw_region = this.draw_region();

        if let Some(para) = &self.paragraph {
            para.paint(content.canvas(), draw_region.0);
            return Ok(());
        }

        let mut pb = ParagraphBuilder::new(
            &get_paragraph_style(this.draw_region()),
            &self.font_collection,
        );

        let mut paragraph = pb
            .push_style(&get_text_style(&self.props.style))
            .add_text(&self.props.text)
            .build();

        paragraph.layout((draw_region.1 .0 - draw_region.0 .0).to_physical());
        paragraph.paint(content.canvas(), draw_region.0);
        self.paragraph = Some(paragraph);
        Ok(())
    }

    fn set_draw_region(&mut self, _: &RcElementModel<Self>, draw_region: Region) {
        let Some(p) = &mut self.paragraph else {
            return;
        };

        p.layout((draw_region.1 .0 - draw_region.0 .0).to_physical());
        if p.height() > (draw_region.1 .1 - draw_region.0 .1).to_physical() {
            self.paragraph.take();
        }
    }
}

fn get_paragraph_style(draw_region: Region) -> ParagraphStyle {
    let mut ps = ParagraphStyle::new();
    ps.set_height((draw_region.1 - draw_region.0).1.to_physical());
    ps
}

fn get_text_style(style: &TextBoxStyles) -> TextStyle {
    let mut text_style = TextStyle::new();
    text_style
        .set_font_style(FontStyle::new(style.weight.0, Width::NORMAL, style.slant.0))
        .set_font_size(style.font_size.0.to_physical())
        .set_color(match &style.color {
            Some(c) => c.0,
            None => Color::BLACK,
        });
    text_style
}

impl<Pr> ElementUpdateRaw<Pr> for TextBox
where
    OwnedProps: PropsUpdateWith<Pr>,
{
    fn el_create(_: &RcElementModel<Self>, props: Pr) -> Self {
        let mut font_collection = FontCollection::new();
        font_collection.set_default_font_manager(FontMgr::new(), None);
        TextBox {
            props: OwnedProps::props_create_with(props),
            font_collection,
            paragraph: None,
            selection: None,
        }
    }

    fn el_update(&mut self, _: &RcElementModel<Self>, props: Pr, _equality_matters: bool) -> bool {
        let update_result = self.props.props_update_with(props);
        if !update_result.unchanged {
            self.paragraph.take();
        }
        update_result.unchanged
    }
}
