use irisia_backend::skia_safe::{Canvas, Color, Color4f, ColorSpace, Paint, RRect, Rect as SkRect};

use crate::primitive::Region;

use super::BlockStyle;

pub(super) struct DrawRRect {
    paint_fill: Option<Paint>,
    paint_stroke: Option<Paint>,
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
            left: draw_region.left_top.x + margin,
            top: draw_region.left_top.y + margin,
            right: draw_region.right_bottom.x - margin,
            bottom: draw_region.right_bottom.y - margin,
        };

        let rrect = RRect::new_nine_patch(
            rect,
            border_radius[0],
            border_radius[1],
            border_radius[2],
            border_radius[3],
        );

        let color_space = ColorSpace::new_srgb_linear();

        let paint_fill = if background != Color::TRANSPARENT {
            let mut paint = Paint::new(&Color4f::from(background), &color_space);
            paint.set_anti_alias(true);
            Some(paint)
        } else {
            None
        };

        let paint_stroke = if border_width != 0.0 {
            let mut paint = Paint::new(&Color4f::from(border_color), &color_space);
            paint.set_stroke(true).set_stroke_width(border_width);
            // .set_stroke_miter();
            paint.set_anti_alias(true);
            Some(paint)
        } else {
            None
        };

        Self {
            paint_fill,
            paint_stroke,
            rrect,
        }
    }

    pub fn draw(&self, canvas: &Canvas) {
        let draw = |paint: &Option<Paint>| {
            if let Some(paint) = paint {
                canvas.draw_rrect(&self.rrect, paint);
            }
        };
        draw(&self.paint_fill);
        draw(&self.paint_stroke);
    }
}
