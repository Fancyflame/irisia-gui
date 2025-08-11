use irisia::{
    primitive::{length::LengthStandard, Length},
    style, Point, Rect, Size,
};
use taffy::{Overflow, Position};

use crate::layouts::{point_to_taffy, rect_to_taffy, size_to_taffy, ResolvedStyle};

#[style(ChildStyleExt)]
#[derive(Clone, Copy, PartialEq)]
pub struct ChildStyle {
    pub width: Length,
    pub height: Length,
    pub min_width: Length,
    pub min_height: Length,
    pub max_width: Length,
    pub max_height: Length,
    pub overflow: Point<Overflow>,
    pub position: Position,
    pub inset: Rect<Length>,
    pub aspect_ratio: Option<f32>,
}

impl ChildStyle {
    pub const DEFAULT: Self = Self {
        width: Length::Auto,
        height: Length::Auto,
        min_width: Length::Auto,
        min_height: Length::Auto,
        max_width: Length::Auto,
        max_height: Length::Auto,
        overflow: Point::all(Overflow::Hidden),
        position: Position::Relative,
        inset: Rect::all(Length::Auto),
        aspect_ratio: None,
    };
}

impl Default for ChildStyle {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl taffy::CoreStyle for ResolvedStyle<'_, &ChildStyle> {
    fn box_generation_mode(&self) -> taffy::BoxGenerationMode {
        taffy::BoxGenerationMode::Normal
    }

    fn is_block(&self) -> bool {
        false
    }

    fn overflow(&self) -> taffy::Point<Overflow> {
        point_to_taffy(self.style.overflow)
    }

    fn position(&self) -> Position {
        self.style.position
    }

    fn inset(&self) -> taffy::Rect<taffy::LengthPercentageAuto> {
        rect_to_taffy(
            self.style
                .inset
                .split_into_hv_components()
                .map_with(self.length_standard.to_point(), |axis, ls| {
                    axis.map(|x| match ls.resolve(x) {
                        Some(value) => taffy::LengthPercentageAuto::length(value),
                        None => taffy::LengthPercentageAuto::auto(),
                    })
                })
                .merge_hv_components(),
        )
    }

    fn size(&self) -> taffy::Size<taffy::Dimension> {
        width_height_to_style_size(self.style.width, self.style.height, self.length_standard)
    }

    fn min_size(&self) -> taffy::Size<taffy::Dimension> {
        width_height_to_style_size(
            self.style.min_width,
            self.style.min_height,
            self.length_standard,
        )
    }

    fn max_size(&self) -> taffy::Size<taffy::Dimension> {
        width_height_to_style_size(
            self.style.max_width,
            self.style.max_height,
            self.length_standard,
        )
    }

    fn aspect_ratio(&self) -> Option<f32> {
        self.style.aspect_ratio
    }
}

fn width_height_to_style_size(
    width: Length,
    height: Length,
    ls: Size<&LengthStandard>,
) -> taffy::Size<taffy::Dimension> {
    size_to_taffy(
        (Size { width, height }).map_with(ls, |x, ls| match ls.resolve(x) {
            Some(value) => taffy::Dimension::length(value),
            None => taffy::Dimension::auto(),
        }),
    )
}
