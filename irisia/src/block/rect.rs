use irisia_backend::skia_safe::{Color, Color4f, Paint, RRect, Rect as SkRect};

use crate::{model2::VModel, primitive::Region};

use super::RenderUnit;

#[derive(Default)]
pub struct Rect {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub color: Color,
    pub border_radius: [f32; 4],
}

impl VModel for Rect {
    type Storage = Self;
    fn create(self, _: &crate::el_model::EMCreateCtx) -> Self::Storage {
        self
    }
    fn update(self, storage: &mut Self::Storage, _: &crate::el_model::EMCreateCtx) {
        *storage = self
    }
}

impl RenderUnit for Rect {
    fn render(&mut self, args: super::RenderArgs, draw_region: Region) {
        if !args.needs_redraw(draw_region) {
            return;
        }

        let rrect = RRect::new_nine_patch(
            SkRect {
                left: draw_region.left_top.0 + self.left,
                top: draw_region.left_top.1 + self.top,
                right: draw_region.right_bottom.0 - self.right,
                bottom: draw_region.right_bottom.1 - self.bottom,
            },
            self.border_radius[0],
            self.border_radius[1],
            self.border_radius[2],
            self.border_radius[3],
        );
        let paint = Paint::new(Color4f::from(self.color), None);

        args.canvas.draw_rrect(rrect, &paint);
    }
}
