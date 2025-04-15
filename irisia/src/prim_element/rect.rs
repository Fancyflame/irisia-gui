use irisia_backend::skia_safe::{Color, Color4f, Paint, RRect, Rect as SkRect};

use crate::primitive::Region;

use super::{
    redraw_guard::RedrawGuard, Common, EMCreateCtx, EmitEventArgs, EventCallback, RenderTree,
};

pub struct RenderRect {
    rect: RectStyle,
    common: Common,
}

#[derive(Default, PartialEq, Clone)]
pub struct RectStyle {
    pub color: Color,
    pub border_radius: [f32; 4],
}

impl RenderRect {
    pub fn new(style: RectStyle, event_callback: Option<EventCallback>, ctx: &EMCreateCtx) -> Self {
        Self {
            rect: style,
            common: Common::new(event_callback, ctx),
        }
    }

    pub fn update_rect(&mut self) -> RedrawGuard<RectStyle> {
        RedrawGuard::new(&mut self.rect, &mut self.common)
    }
}

impl RenderTree for RenderRect {
    fn render(&mut self, args: super::RenderArgs, draw_region: Region) {
        if !args.needs_redraw(draw_region) {
            return;
        }

        self.common.prev_draw_region = Some(draw_region);
        let border_radius = self.rect.border_radius;
        let rrect = RRect::new_nine_patch(
            SkRect {
                left: draw_region.left_top.0,
                top: draw_region.left_top.1,
                right: draw_region.right_bottom.0,
                bottom: draw_region.right_bottom.1,
            },
            border_radius[0],
            border_radius[1],
            border_radius[2],
            border_radius[3],
        );
        let paint = Paint::new(Color4f::from(self.rect.color), None);

        args.canvas.draw_rrect(rrect, &paint);
    }

    fn emit_event(&mut self, args: EmitEventArgs) {
        self.common.use_callback(args);
    }

    fn set_callback(&mut self, callback: EventCallback) {
        self.common.event_callback = Some(callback);
    }
}
