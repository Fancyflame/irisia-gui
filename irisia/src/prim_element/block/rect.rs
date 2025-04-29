use irisia_backend::skia_safe::{Canvas, Color4f, ColorSpace, Paint, RRect, Rect as SkRect};

use crate::primitive::Region;

use super::BlockStyle;

pub(super) struct DrawRRect {
    paint_fill: Paint,
    paint_stroke: Paint,
    rrect: RRect,
}

impl DrawRRect {
    pub fn new(style: &BlockStyle, draw_region: Region) -> Self {
        let &BlockStyle {
            margin,
            background,
            border_width,
            border_color,
            border_radius,
        } = style;

        let rect = SkRect {
            left: draw_region.left_top.0 + margin,
            top: draw_region.left_top.1 + margin,
            right: draw_region.right_bottom.0 - margin,
            bottom: draw_region.right_bottom.1 - margin,
        };

        let rrect = RRect::new_nine_patch(
            rect,
            border_radius[0],
            border_radius[1],
            border_radius[2],
            border_radius[3],
        );

        let color_space = ColorSpace::new_srgb_linear();

        let mut paint_fill = Paint::new(&Color4f::from(background), &color_space);
        paint_fill.set_anti_alias(true);

        let mut paint_stroke = Paint::new(&Color4f::from(border_color), &color_space);
        paint_stroke.set_stroke(true).set_stroke_width(border_width);
        // .set_stroke_miter();
        paint_stroke.set_anti_alias(true);

        Self {
            paint_fill,
            paint_stroke,
            rrect,
        }
    }

    pub fn draw(&self, canvas: &Canvas) {
        canvas
            .draw_rrect(&self.rrect, &self.paint_fill)
            .draw_rrect(&self.rrect, &self.paint_stroke);
    }
}
