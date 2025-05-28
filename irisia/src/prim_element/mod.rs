use std::{
    cell::RefCell,
    ops::Deref,
    rc::{Rc, Weak},
    time::Duration,
};

use block::RenderBlock;
use callback_queue::CallbackQueue;
use irisia_backend::skia_safe::{Canvas, ClipOp, Region as SkRegion};
use layout::{FinalLayout, LayoutInput};

use crate::{
    Handle, WeakHandle,
    application::{
        content::GlobalContent,
        event2::pointer_event::{PointerEvent, PointerStateDelta},
    },
    hook::{Signal, utils::trace_cell::TraceRef},
    primitive::{Point, Rect, Region, size::Size},
};

pub(crate) use common::Common;
pub mod block;
pub(crate) mod callback_queue;
mod common;
pub mod layout;
// pub mod image;
mod redraw_guard;
pub mod text;

pub(crate) type EventCallback = Signal<dyn Fn(PointerEvent)>;
pub(crate) type Parent = Option<Weak<RefCell<RenderBlock>>>;

#[derive(Clone)]
pub struct EMCreateCtx {
    pub(crate) global_content: Rc<GlobalContent>,
    pub(crate) parent: Parent,
}

pub trait RenderTree: 'static {
    fn render(&mut self, args: RenderArgs, draw_location: Point<f32>);
    fn compute_layout(&mut self, inputs: LayoutInput) -> Size<f32>;
    fn children_emit_event(&mut self, args: &mut EmitEventArgs);
    fn common_mut(&mut self) -> &mut Common;
    fn common(&self) -> &Common;
}

impl<T: RenderTree + ?Sized> RenderTreeExt for T {}
pub(crate) trait RenderTreeExt: RenderTree {
    fn render_entry(&mut self, args: RenderArgs, parent_location: Point<f32>) {
        let final_layout = self.common_mut().layout_output;
        self.common_mut().prev_draw_region = Some(final_layout.as_rect());

        if final_layout.is_hidden() {
            return;
        }

        let absolute_draw_region = final_layout.as_rect() + parent_location.split_hv_to_rect();

        if args.needs_redraw(absolute_draw_region) {
            self.render(args, absolute_draw_region.get_location());
        }
    }

    fn emit_event(&mut self, args: &mut EmitEventArgs) {
        self.children_emit_event(args);
        self.common_mut().use_callback(args);
    }

    fn set_callback(&mut self, callback: EventCallback) {
        self.common_mut().event_callback = Some(callback);
    }

    fn compute_layout_cached(&mut self, inputs: LayoutInput) -> Size<f32> {
        match self.common().layout_input {
            Some(old_inputs) if old_inputs == inputs => self.common().layout_output.size,
            _ => self.force_compute_layout_cached(inputs),
        }
    }

    fn force_compute_layout_cached(&mut self, inputs: LayoutInput) -> Size<f32> {
        let computed_size = self.compute_layout(inputs);
        let common = self.common_mut();
        common.layout_input = Some(inputs);
        common.layout_output.size = computed_size;
        computed_size
    }

    fn set_layout_completed(&mut self) {
        let common = self.common_mut();
        let new_layout = common.layout_output;

        if matches!(common.prev_draw_region, Some(prev) if prev == new_layout.as_rect())
            || (common.layout_output.is_hidden() && new_layout.is_hidden())
        {
            return;
        }

        common.request_repaint();
    }

    fn clear_layout_cache(&mut self) {
        self.common_mut().layout_input.take();
        self.common_mut().layout_output = FinalLayout::HIDDEN;
    }
}

#[derive(Clone, Copy)]
pub struct RenderArgs<'a> {
    pub canvas: &'a Canvas,
    pub interval: Duration,
    pub dirty_region: Option<&'a SkRegion>,
}

impl RenderArgs<'_> {
    pub fn needs_redraw(&self, draw_region: Rect<f32>) -> bool {
        let draw_rect = draw_region.round_to_skia_irect();
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
pub type WeakElement = WeakHandle<dyn RenderTree>;

pub struct EmitEventArgs<'a> {
    pub(crate) queue: &'a mut CallbackQueue,
    pub(crate) delta: PointerStateDelta,
}

fn make_region(location: Point, width: f32, height: f32) -> Region {
    Region {
        left_top: location,
        right_bottom: Point {
            x: location.x + width,
            y: location.y + height,
        },
    }
}

fn clip_draw_region(canvas: &Canvas, region: Region) {
    let rect = region.ceil_to_irect();
    canvas.clip_region(&SkRegion::from_rect(rect), ClipOp::Intersect);
}

fn read_or_default<'a, T: ?Sized>(
    sig: &'a Option<Signal<T>>,
    default: &'a T,
) -> impl Deref<Target = T> + use<'a, T> {
    enum Ref<'a, T: ?Sized> {
        Ref(&'a T),
        TRef(TraceRef<'a, T>),
    }

    impl<T: ?Sized> Deref for Ref<'_, T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            match self {
                Self::Ref(r) => &r,
                Self::TRef(r) => &r,
            }
        }
    }

    match sig {
        Some(sig) => Ref::TRef(sig.read()),
        None => Ref::Ref(default),
    }
}
