use std::{
    cell::{Cell, RefCell},
    sync::Arc,
    time::Duration,
};

use irisia_backend::{
    WinitWindow,
    skia_safe::{Canvas, Color, Region as SkRegion, region::RegionOp},
    winit::dpi::PhysicalSize,
};

use crate::{
    prim_element::{Element, RenderArgs, RenderTree, RenderTreeExt},
    primitive::{Point, Region},
};

use super::window_size_to_constraint;

pub(super) struct RedrawScheduler {
    window: Arc<WinitWindow>,
    dirty_region: RefCell<SkRegion>,
    redraw_req_sent: Cell<bool>,
    relayout_mode: Cell<bool>,
}

impl RedrawScheduler {
    pub fn new(window: Arc<WinitWindow>) -> Self {
        Self {
            window,
            dirty_region: RefCell::new(SkRegion::new()),
            redraw_req_sent: Cell::new(false),
            relayout_mode: Cell::new(true),
        }
    }

    fn request_window_redraw(&self) {
        if !self.redraw_req_sent.get() {
            self.redraw_req_sent.set(true);
            self.window.request_redraw();
        }
    }

    pub fn request_relayout(&self) {
        self.request_window_redraw();
        self.relayout_mode.set(true);
    }

    pub fn request_redraw(&self, region: Region) {
        self.request_window_redraw();
        self.dirty_region
            .borrow_mut()
            .op_rect(region.ceil_to_irect(), RegionOp::Union);
    }

    pub fn redraw(
        &self,
        root: &Element,
        canvas: &Canvas,
        interval: Duration,
        draw_size: PhysicalSize<u32>,
    ) {
        let mut root = root.borrow_mut();

        if self.relayout_mode.take() {
            root.layout(window_size_to_constraint(draw_size));
        }

        // TODO: operate dirty region
        let _dirty_region = self.dirty_region.replace(SkRegion::new());

        self.redraw_req_sent.set(false);
        canvas.save();
        //canvas.clip_region(&redrawing_dirty_region, ClipOp::Intersect);
        canvas.clear(Color::WHITE);
        root.render_entry(
            RenderArgs {
                canvas,
                interval,
                dirty_region: None,
            },
            Point::ZERO,
        );
        canvas.restore();
    }
}
