use irisia_backend::skia_safe::{
    Canvas, ClipOp, Color, Color4f, ColorSpace, Paint, Point as SkPoint, RRect, Rect as SkRect,
};

use crate::primitive::{Point, corner::Corner, rect::Rect, size::Size};

pub(super) struct DrawRRect {
    fill_content: Option<Paint>,
    fill_border_content: Option<Paint>,
    border_content_rrect: RRect,
    content_rrect: RRect,
}

pub(super) struct DrawRRectProps {
    pub background: Color,
    pub border_color: Color,
    pub border_radius: Corner<f32>,
    pub margin: Rect<f32>,
    pub border: Rect<f32>,
    pub outer_size: Size<f32>,
}

impl DrawRRect {
    pub fn new(props: DrawRRectProps) -> Self {
        let DrawRRectProps {
            background,
            border_color,
            border_radius,
            margin,
            border,
            outer_size,
        } = props;

        let border_content_rect = SkRect {
            left: margin.left,
            top: margin.top,
            right: outer_size.width - margin.right,
            bottom: outer_size.height - margin.bottom,
        };

        let content_rect = SkRect {
            left: border_content_rect.left + border.left,
            top: border_content_rect.top + border.top,
            right: border_content_rect.right - border.right,
            bottom: border_content_rect.bottom - border.bottom,
        };

        let make_rrect = |rect: SkRect| {
            let arr = [
                border_radius.left_top,
                border_radius.right_top,
                border_radius.right_bottom,
                border_radius.left_bottom,
            ]
            .map(|x| SkPoint::new(x, x));

            RRect::new_rect_radii(rect, &arr)
        };

        let color_space = ColorSpace::new_srgb_linear();

        let fill_content = if background != Color::TRANSPARENT {
            let mut paint = Paint::new(&Color4f::from(background), &color_space);
            paint.set_anti_alias(true);
            Some(paint)
        } else {
            None
        };

        let fill_border_content = if border != Rect::all(0.0) {
            let mut paint = Paint::new(&Color4f::from(border_color), &color_space);
            paint.set_anti_alias(true);
            Some(paint)
        } else {
            None
        };

        Self {
            fill_content,
            fill_border_content,
            border_content_rrect: make_rrect(border_content_rect),
            content_rrect: make_rrect(content_rect),
        }
    }

    pub fn draw(&self, canvas: &Canvas, location: Point<f32>) {
        let location = SkPoint::new(location.x, location.y);
        canvas.save();
        canvas.translate(location);

        if let Some(fill_border_content) = &self.fill_border_content {
            canvas.clip_rrect(self.content_rrect, ClipOp::Intersect, true);
            canvas.draw_rrect(self.border_content_rrect, fill_border_content);
        }

        if let Some(fill_content) = &self.fill_content {
            canvas.draw_rrect(self.content_rrect, fill_content);
        }

        canvas.restore();
    }
}
