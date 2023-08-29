use std::ops::Range;

use irisia::{
    element::Element,
    skia_safe::{
        font_style::Width,
        textlayout::{FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, TextStyle},
        Color, FontMgr, FontStyle, Point as SkiaPoint,
    },
    style::{StyleColor, StyleContainer},
    StyleReader,
};
use irisia::{
    primitive::Region,
    skia_safe::{Color4f, ColorSpace, Paint},
};
use styles::*;

use crate::box_styles::BoxStyleRenderer;

use self::selection::SelectionRtMgr;

mod selection;
pub mod styles;

pub struct TextBox {
    font_collection: FontCollection,
    paragraph: Option<Paragraph>,
    previous_state: Option<OwnedState>,
    selection: Option<Range<usize>>,
    selection_rt_mgr: SelectionRtMgr,
}

#[derive(StyleReader, PartialEq)]
struct TextBoxStyles {
    font_size: StyleFontSize,
    slant: StyleFontSlant,
    weight: StyleFontWeight,
    color: Option<StyleColor>,
}

#[irisia::props(Props, "pub")]
pub struct TextBoxOwnedProps {
    #[props(updated, must_init)]
    pub text: String,

    #[props(default = "false")]
    pub user_select: bool,
}

#[derive(PartialEq)]
struct OwnedState {
    text: String,
    styles: TextBoxStyles,
    user_select: bool,
    drawing_region: Region,
}
