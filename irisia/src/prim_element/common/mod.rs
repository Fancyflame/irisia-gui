use super::{
    EMCreateCtx, EmitEventArgs, EventCallback, Size,
    layout::{FinalLayout, SpaceConstraint},
};
use crate::primitive::Region;

pub(super) struct Common {
    prev_cursor_over: bool,
    pub ctx: EMCreateCtx,
    pub event_callback: Option<EventCallback>,
    pub cached_layout: Option<(Size<SpaceConstraint>, Size<f32>)>,
    pub final_layout: Option<FinalLayout>,
    pub redraw_request_sent: bool,
}

impl Common {
    pub fn new(event_callback: Option<EventCallback>, ctx: &EMCreateCtx) -> Common {
        let mut this = Self {
            prev_cursor_over: false,
            event_callback,
            final_layout: None,
            cached_layout: None,
            ctx: ctx.clone(),
            redraw_request_sent: false,
        };
        this.request_redraw();
        this
    }

    pub fn use_callback(&mut self, args: &mut EmitEventArgs) {
        let Some(final_layout) = self.final_layout else {
            return;
        };

        let draw_region = final_layout.region.to_lagacy_region();

        let events = args
            .delta
            .get_event(draw_region, &mut self.prev_cursor_over);

        if let Some(sig) = &self.event_callback {
            args.queue.extend(events.map(|pe| (sig.clone(), pe)));
        }
    }

    pub fn request_redraw(&mut self) {
        if self.redraw_request_sent {
            return;
        }

        if let Some(final_layout) = self.final_layout {
            self.ctx
                .global_content
                .request_redraw(final_layout.region.to_lagacy_region());
        }

        // TODO: 设置脏区
        self.ctx.global_content.request_redraw(Region::default());

        self.redraw_request_sent = true;
    }

    pub fn request_relayout(&mut self) {
        self.cached_layout.take();
        if let Some(parent) = &self.ctx.parent {
            parent
                .upgrade()
                .expect("parent should not be dropped when child is alive")
                .borrow_mut()
                .set_children_size_changed();
        } else {
            self.ctx.global_content.request_relayout();
        }
    }
}
