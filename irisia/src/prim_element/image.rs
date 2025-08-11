use irisia_backend::skia_safe::{Color, Color4f, Image, Paint, RRect, Rect as SkRect};

use crate::primitive::Region;

use super::{
    redraw_guard::RedrawGuard, Common, EMCreateCtx, EmitEventArgs, EventCallback, RenderTree,
};

pub struct RenderImage {
    style: ImageStyle,
    image: Option<Image>,
    common: Common,
}

#[derive(Default, PartialEq, Clone)]
pub struct ImageStyle {
    pub color: Color,
    pub border_radius: [f32; 4],
}

impl RenderImage {
    pub fn new(
        style: ImageStyle,
        image: Option<Image>,
        event_callback: Option<EventCallback>,
        ctx: &EMCreateCtx,
    ) -> Self {
        Self {
            style,
            image,
            common: Common::new(event_callback, ctx),
        }
    }

    pub fn update_style(&mut self) -> RedrawGuard<ImageStyle> {
        RedrawGuard::new(&mut self.style, &mut self.common)
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
