use super::{
    EMCreateCtx, EmitEventArgs, EventCallback, RenderTree, WeakElement,
    layout::{FinalLayout, LayoutInput},
};
use crate::{WeakHandle, primitive::Rect};

pub struct Common {
    prev_cursor_over: bool,
    element: WeakElement,
    pub ctx: EMCreateCtx,
    pub prev_draw_region: Option<Rect<f32>>,
    pub event_callback: Option<EventCallback>,
    pub layout_input: Option<LayoutInput>,
    pub layout_output: FinalLayout,
}

impl Common {
    pub fn new(
        el: WeakHandle<dyn RenderTree>,
        event_callback: Option<EventCallback>,
        ctx: &EMCreateCtx,
    ) -> Common {
        Self {
            prev_cursor_over: false,
            prev_draw_region: None,
            element: el,
            event_callback,
            layout_output: FinalLayout::HIDDEN,
            layout_input: None,
            ctx: ctx.clone(),
        }
    }

    pub fn use_callback(&mut self, args: &mut EmitEventArgs) {
        // if self.layout_output.is_hidden() {
        //     return;
        // }

        let draw_region = self.layout_output.as_rect().to_lagacy_region();

        let events = args
            .delta
            .get_event(draw_region, &mut self.prev_cursor_over);

        if let Some(sig) = &self.event_callback {
            args.queue.extend(events.map(|pe| (sig.clone(), pe)));
        }
    }

    pub fn request_repaint(&self) {
        self.ctx.global_content.request_repaint(&self.element);
    }

    pub fn request_reflow(&self) {
        self.ctx.global_content.request_reflow(&self.element);
    }
}
