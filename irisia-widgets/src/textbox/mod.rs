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

mod el;
mod selection;
pub mod styles;

pub struct TextBox {
    props: OwnedProps,
    font_collection: FontCollection,
    paragraph: Option<Paragraph>,
    selection: Option<Range<usize>>,
    selection_rt_mgr: SelectionRtMgr,
}

#[derive(PartialEq)]
struct OwnedState {
    text: String,
    styles: TextBoxStyles,
    user_select: bool,
    drawing_region: Region,
}

#[derive(StyleReader, PartialEq)]
struct TextBoxStyles {
    font_size: StyleFontSize,
    slant: StyleFontSlant,
    weight: StyleFontWeight,
    color: Option<StyleColor>,
}

#[irisia::props(Props, vis = "pub")]
pub struct OwnedProps {
    #[props(updated, must_init)]
    text: String,

    #[props(read_style(stdin))]
    style: TextBoxStyles,

    #[props(default = "false")]
    user_select: bool,
}
