use std::{
    cell::{Cell, RefCell},
    sync::Arc,
    time::Duration,
};

use irisia_backend::{
    skia_safe::{region::RegionOp, Canvas, Color, Region as SkRegion},
    WinitWindow,
};

use crate::{
    el_model::ElementModel,
    element::Render,
    prim_element::{Element, RenderArgs, RenderTree},
    primitive::Region,
    ElementInterfaces, Result,
};

pub(super) struct RedrawScheduler {
    window: Arc<WinitWindow>,
    dirty_region: RefCell<SkRegion>,
    redrawing_dirty_region: RefCell<SkRegion>,
    redraw_req_sent: Cell<bool>,
}

impl RedrawScheduler {
    pub fn new(window: Arc<WinitWindow>) -> Self {
        Self {
            window,
            dirty_region: RefCell::new(SkRegion::new()),
            redrawing_dirty_region: RefCell::new(SkRegion::new()),
            redraw_req_sent: Cell::new(false),
        }
    }

    pub fn request_redraw(&self, dirty_region: Region) {
        if !self.redraw_req_sent.get() {
            self.redraw_req_sent.set(true);
            self.window.request_redraw();
        }
        self.dirty_region.borrow_mut().op_region(
            &SkRegion::from_rect(dirty_region.ceil_to_irect()),
            RegionOp::Union,
        );
    }

    pub fn redraw(
        &self,
        root: &mut Element,
        canvas: &Canvas,
        interval: Duration,
        root_draw_region: Region,
    ) {
        let mut redrawing_dirty_region = self.redrawing_dirty_region.borrow_mut();
        redrawing_dirty_region.set_empty();
        std::mem::swap(
            &mut *redrawing_dirty_region,
            &mut *self.dirty_region.borrow_mut(),
        );

        self.redraw_req_sent.set(false);
        canvas.save();
        //canvas.clip_region(&dirty_region, ClipOp::Intersect);
        canvas.clear(Color::WHITE);
        root.render(
            RenderArgs {
                canvas,
                interval,
                dirty_region: None,
            },
            root_draw_region,
        );
        canvas.restore();
    }
}
