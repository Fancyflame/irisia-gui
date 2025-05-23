use std::{
    cell::RefCell,
    ops::Deref,
    rc::{Rc, Weak},
    time::Duration,
};

use block::RenderBlock;
use callback_queue::CallbackQueue;
use irisia_backend::skia_safe::{Canvas, ClipOp, Region as SkRegion};
use layout::FinalLayout;

use crate::{
    Handle,
    application::{
        content::GlobalContent,
        event2::pointer_event::{PointerEvent, PointerStateDelta},
    },
    hook::{Signal, utils::trace_cell::TraceRef},
    primitive::{Point, Region, length::LengthStandard, size::Size},
};

pub(self) use common::Common;
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
    fn compute_layout(
        &mut self,
        constraint: Size<layout::SpaceConstraint>,
        length_standard: Size<LengthStandard>,
    ) -> Size<f32>;
    fn children_emit_event(&mut self, args: &mut EmitEventArgs);
    fn common_mut(&mut self) -> &mut Common;
}

impl<T: RenderTree + ?Sized> RenderTreeExt for T {}
pub(crate) trait RenderTreeExt: RenderTree {
    fn render_entry(&mut self, args: RenderArgs, parent_location: Point<f32>) {
        self.common_mut().redraw_request_sent = false;

        let absolute_draw_region = match self.common_mut().final_layout {
            Some(visible_layout) => visible_layout.region + parent_location.split_hv_to_rect(),
            None => return,
        };

        if args.needs_redraw(absolute_draw_region.to_lagacy_region()) {
            self.render(args, absolute_draw_region.get_location());
        }
    }

    fn emit_event(&mut self, args: &mut EmitEventArgs) {
        self.common_mut().use_callback(args);
        self.children_emit_event(args);
    }

    fn set_callback(&mut self, callback: EventCallback) {
        self.common_mut().event_callback = Some(callback);
    }

    fn compute_layout_cached(
        &mut self,
        constraint: Size<layout::SpaceConstraint>,
        length_standard: Size<LengthStandard>,
    ) -> Size<f32> {
        match &self.common_mut().cached_layout {
            Some((cached_constraint, cached)) if *cached_constraint == constraint => *cached,
            _ => {
                let computed = self.compute_layout(constraint, length_standard);
                self.common_mut().cached_layout = Some((constraint, computed));
                computed
            }
        }
    }

    fn set_final_layout(&mut self, new_layout: Option<FinalLayout>) {
        let common = self.common_mut();

        if common.final_layout == new_layout {
            return;
        }

        common.final_layout = new_layout;
        common.request_redraw();
    }
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
