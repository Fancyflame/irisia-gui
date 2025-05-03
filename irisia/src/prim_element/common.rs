use super::{EMCreateCtx, EmitEventArgs, EventCallback, Size, SpaceConstraint, make_region};
use crate::primitive::{Point, Region};

pub(super) struct Common {
    prev_cursor_over: bool,
    pub ctx: EMCreateCtx,
    pub event_callback: Option<EventCallback>,
    pub cached_layout: Option<(Size<SpaceConstraint>, Size<f32>)>,
    prev_draw_region: Option<Region>,
    redraw_request_sent: bool,
}

impl Common {
    pub fn new(event_callback: Option<EventCallback>, ctx: &EMCreateCtx) -> Common {
        let mut this = Self {
            prev_cursor_over: false,
            event_callback,
            prev_draw_region: None,
            cached_layout: None,
            ctx: ctx.clone(),
            redraw_request_sent: false,
        };
        this.request_redraw();
        this
    }

    pub fn use_callback(&mut self, args: EmitEventArgs) {
        let Some(draw_region) = self.prev_draw_region else {
            return;
        };

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

        if let Some(prev_dr) = self.prev_draw_region {
            self.ctx.global_content.request_redraw(prev_dr);
        }

        // TODO: 设置脏区
        self.ctx.global_content.request_redraw(Region::default());

        self.redraw_request_sent = true;
    }

    pub fn request_relayout(&mut self) {
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

    pub fn set_rendered(&mut self, location: Point) -> Region {
        let (_, size) = self
            .cached_layout
            .expect("element cannot be rendered before layout");
        let new_region = make_region(location, size.width, size.height);
        self.prev_draw_region = Some(new_region);
        self.redraw_request_sent = false;
        new_region
    }

    pub fn use_cached_layout<F>(
        &mut self,
        input_constraint: Size<SpaceConstraint>,
        force_compute: bool,
        compute: F,
    ) -> Size<f32>
    where
        F: FnOnce(Size<SpaceConstraint>) -> Size<f32>,
    {
        if let (Some((constraint, result)), false) = (self.cached_layout, force_compute) {
            if constraint == input_constraint {
                return result;
            }
        }

        let result = compute(input_constraint);
        self.cached_layout = Some((input_constraint, result));
        result
    }
}
