use std::{cell::RefCell, rc::Rc, time::Duration};

use callback_queue::CallbackQueue;
use irisia_backend::skia_safe::{Canvas, Region as SkRegion};

use crate::{
    application::{
        content::GlobalContent,
        event2::pointer_event::{PointerEvent, PointerStateDelta},
    },
    hook::Signal,
    primitive::Region,
};

pub mod block;
pub(crate) mod callback_queue;
pub mod rect;
mod redraw_guard;
pub mod text;

type Handle<T> = Rc<RefCell<T>>;
pub(crate) type EventCallback = Signal<dyn Fn(PointerEvent)>;

#[derive(Clone)]
pub struct EMCreateCtx {
    pub(crate) global_content: Rc<GlobalContent>,
}

pub trait RenderTree: 'static {
    fn render(&mut self, args: RenderArgs, draw_region: Region);
    fn emit_event(&mut self, args: EmitEventArgs);
    fn set_callback(&mut self, callback: EventCallback);
}

#[derive(Clone, Copy)]
pub struct RenderArgs<'a> {
    pub canvas: &'a Canvas,
    pub interval: Duration,
    pub dirty_region: Option<&'a SkRegion>,
}

impl RenderArgs<'_> {
    pub fn needs_redraw(&self, draw_region: Region) -> bool {
        let draw_rect = draw_region.ceil_to_irect();
        if let Some(dirty_region) = self.dirty_region {
            if dirty_region.quick_reject_rect(draw_rect) {
                return false;
            }
            if !dirty_region.intersects_rect(draw_rect) {
                return false;
            }
        }
        true
    }
}

pub type Element = Handle<dyn RenderTree>;

impl RenderTree for Element {
    fn render(&mut self, args: RenderArgs, draw_region: Region) {
        self.borrow_mut().render(args, draw_region);
    }
    fn emit_event(&mut self, args: EmitEventArgs) {
        self.borrow_mut().emit_event(args);
    }
    fn set_callback(&mut self, callback: EventCallback) {
        self.borrow_mut().set_callback(callback);
    }
}

struct Common {
    prev_cursor_over: bool,
    event_callback: Option<EventCallback>,
    prev_draw_region: Option<Region>,
    ctx: EMCreateCtx,
}

impl Common {
    fn new(event_callback: Option<EventCallback>, ctx: &EMCreateCtx) -> Common {
        Self {
            prev_cursor_over: false,
            event_callback,
            prev_draw_region: None,
            ctx: ctx.clone(),
        }
    }

    fn use_callback(&mut self, args: EmitEventArgs) {
        let events = args
            .delta
            .get_event(args.draw_region, &mut self.prev_cursor_over);

        if let Some(sig) = &self.event_callback {
            args.queue.extend(events.map(|pe| (sig.clone(), pe)));
        }
    }

    fn request_redraw(&mut self) {
        if let Some(dr) = self.prev_draw_region.take() {
            self.ctx.global_content.request_redraw(dr);
        }
    }
}

pub struct EmitEventArgs<'a> {
    pub(crate) queue: &'a mut CallbackQueue,
    pub(crate) delta: &'a mut PointerStateDelta,
    pub(crate) draw_region: Region,
}

impl EmitEventArgs<'_> {
    pub(crate) fn reborrow<'r>(&'r mut self, child_draw_region: Region) -> EmitEventArgs<'r> {
        EmitEventArgs {
            queue: self.queue,
            delta: self.delta,
            draw_region: child_draw_region,
        }
    }
}
