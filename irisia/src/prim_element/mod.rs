use std::{
    cell::RefCell,
    ops::Deref,
    rc::{Rc, Weak},
    time::Duration,
};

use block::RenderBlock;
use callback_queue::CallbackQueue;
use irisia_backend::skia_safe::{Canvas, ClipOp, Region as SkRegion};

use crate::{
    application::{
        content::GlobalContent,
        event2::pointer_event::{PointerEvent, PointerStateDelta},
    },
    hook::{utils::trace_cell::TraceRef, Signal},
    primitive::{Point, Region},
};

pub(self) use common::Common;

pub mod block;
pub(crate) mod callback_queue;
mod common;
// pub mod image;
mod redraw_guard;
pub mod text;

type Handle<T> = Rc<RefCell<T>>;
pub(crate) type EventCallback = Signal<dyn Fn(PointerEvent)>;
pub(crate) type Parent = Option<Weak<RefCell<RenderBlock>>>;

#[derive(Clone)]
pub struct EMCreateCtx {
    pub(crate) global_content: Rc<GlobalContent>,
    pub(crate) parent: Parent,
}

pub trait RenderTree: 'static {
    fn render(&mut self, args: RenderArgs, draw_location: Point);
    fn emit_event(&mut self, args: EmitEventArgs);
    fn layout(&mut self, constraint: Size<SpaceConstraint>) -> Size<f32>;
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
    fn render(&mut self, args: RenderArgs, location: Point) {
        self.borrow_mut().render(args, location);
    }
    fn emit_event(&mut self, args: EmitEventArgs) {
        self.borrow_mut().emit_event(args);
    }
    fn layout(&mut self, constraint: Size<SpaceConstraint>) -> Size<f32> {
        self.borrow_mut().layout(constraint)
    }
    fn set_callback(&mut self, callback: EventCallback) {
        self.borrow_mut().set_callback(callback);
    }
}

pub struct EmitEventArgs<'a> {
    pub(crate) queue: &'a mut CallbackQueue,
    pub(crate) delta: &'a mut PointerStateDelta,
}

impl EmitEventArgs<'_> {
    pub(crate) fn reborrow<'r>(&'r mut self) -> EmitEventArgs<'r> {
        EmitEventArgs {
            queue: self.queue,
            delta: self.delta,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum SpaceConstraint {
    Exact(f32),
    Available(f32),
    MinContent,
    MaxContent,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Size<T> {
    pub x: T,
    pub y: T,
}

fn make_region(location: Point, width: f32, height: f32) -> Region {
    Region {
        left_top: location,
        right_bottom: Point(location.0 + width, location.1 + height),
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
