use super::styles::*;
use irisia::{
    element::{props::PropsUpdateWith, Element, UpdateElement},
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

use super::{selection::SelectionRtMgr, OwnedProps, TextBox};

impl Element for TextBox {
    type BlankProps = super::Props;
}

impl<Pr> UpdateWith<UpdateElement<'_, Self, Pr>> for TextBox
where
    OwnedProps: UpdateWith<Pr>,
{
    fn create_with(updater: UpdateElement<'_, Self, Pr>) -> Self {
        let mut font_collection = FontCollection::new();
        font_collection.set_default_font_manager(FontMgr::new(), None);
        TextBox {
            props: OwnedProps::create_with(updater.props),
            font_collection,
            paragraph: Paragraph::,
            selection: None,
            selection_rt_mgr: SelectionRtMgr::new((*updater.handle).clone()),
        }
    }

    fn update_with(
        &mut self,
        updater: UpdateElement<'_, Self, Pr>,
        equality_matters: bool,
    ) -> bool {
        let update_result = self.props.props_update_with(updater.props);
    }
}
