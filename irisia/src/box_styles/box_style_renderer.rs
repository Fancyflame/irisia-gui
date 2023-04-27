use irisia::{
    primary::{Point, Region},
    read_style,
    skia_safe::{Canvas, Rect},
    style::StyleContainer,
};

use crate::box_styles::{
    border::{draw_border, StyleBorder},
    border_radius::{parse_border_radius, StyleBorderRadius},
    box_shadow::draw_shadow,
};

use super::{box_shadow::StyleBoxShadow, margin::StyleMargin};

pub struct BoxStyleRenderer;

#[derive(Default)]
struct BoundReduction {
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
}

impl BoxStyleRenderer {
    pub fn draw_border_limited(
        styles: &impl StyleContainer,
        canvas: &mut Canvas,
        maximum_region: Region,
    ) -> Region {
        let BoundReduction {
            top,
            right,
            bottom,
            left,
        } = Self::render(styles, canvas, maximum_region);
        (
            maximum_region.0 + Point(left as _, top as _),
            maximum_region.1 - Point(right as _, bottom as _),
        )
    }

    pub fn draw_border_unlimited(
        styles: &impl StyleContainer,
        canvas: &mut Canvas,
        content_region: Region,
    ) -> Region {
        let BoundReduction {
            top,
            right,
            bottom,
            left,
        } = Self::render(styles, canvas, content_region);
        (
            content_region.0 - Point(left as _, top as _),
            content_region.1 + Point(right as _, bottom as _),
        )
    }

    fn render(styles: &impl StyleContainer, canvas: &mut Canvas, region: Region) -> BoundReduction {
        read_style!(styles in styles => {
            border: Option<StyleBorder>,
            radius: StyleBorderRadius,
            box_shadow: Option<StyleBoxShadow>,
            margin: StyleMargin,
        });

        let mut reduction = BoundReduction::default();

        let rect = {
            let StyleMargin {
                top,
                right,
                bottom,
                left,
            } = styles.margin;

            let (left, top, right, bottom) = (
                left.to_physical(),
                top.to_physical(),
                right.to_physical(),
                bottom.to_physical(),
            );

            reduction.left += left;
            reduction.top += top;
            reduction.right += right;
            reduction.bottom += bottom;

            Rect::new(
                region.0 .0 as f32 + left,
                region.0 .1 as f32 + top,
                region.1 .0 as f32 - right,
                region.1 .1 as f32 - bottom,
            )
        };

        let rrect = parse_border_radius(&rect, &styles.radius);

        if let Some(bs) = &styles.box_shadow {
            draw_shadow(canvas, &rrect, bs);
        }

        if let Some(border) = &styles.border {
            let width = draw_border(canvas, rrect, border);
            reduction.left += width;
            reduction.top += width;
            reduction.right += width;
            reduction.bottom += width;
        }

        reduction
    }
}
