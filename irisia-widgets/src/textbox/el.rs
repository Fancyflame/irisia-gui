use super::styles::*;
use irisia::{
    element::{
        props::PropsUpdateWith, AsChildren, Element, ElementUpdate, RcElementModel, RenderElement,
    },
    primitive::Region,
    skia_safe::{
        font_style::Width,
        textlayout::{FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, TextStyle},
        Color, FontMgr, FontStyle, Point as SkiaPoint,
    },
    skia_safe::{Color4f, ColorSpace, Paint},
    style::{StyleColor, StyleContainer},
    StyleReader, UpdateWith,
};

use crate::box_styles::BoxStyleRenderer;

use super::{/*selection::SelectionRtMgr,*/ OwnedProps, TextBox};

impl Element for TextBox {
    type BlankProps = super::TextBoxProps;
    fn render(&mut self, content: RenderElement) -> irisia::Result<()> {
        todo!()
    }
}

impl<Pr> ElementUpdate<Pr> for TextBox
where
    OwnedProps: UpdateWith<Pr>,
{
    fn el_create(
        model: &RcElementModel<Self, impl StyleContainer, impl AsChildren>,
        props: Pr,
    ) -> Self {
        let mut font_collection = FontCollection::new();
        font_collection.set_default_font_manager(FontMgr::new(), None);
        TextBox {
            props: OwnedProps::props_create_with(props),
            font_collection,
            paragraph: None,
            selection: None,
            //selection_rt_mgr: SelectionRtMgr::new(model),
        }
    }

    fn el_update(&mut self, props: Pr, equality_matters: bool) -> bool {
        let update_result = self.props.props_update_with(updater.props);
        if update_result.unchanged {
            return true;
        }

        let builder = ParagraphBuilder::new()
    }
}
