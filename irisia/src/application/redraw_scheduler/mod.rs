use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    sync::Arc,
    time::Duration,
};

use irisia_backend::{
    WinitWindow,
    skia_safe::{Canvas, ClipOp, Color, Region as SkRegion, region::RegionOp},
};
use reflow::ReflowScheduler;

use crate::{
    WeakHandle,
    prim_element::{
        Element, RenderArgs, RenderTree, RenderTreeExt, WeakElement, layout::LayoutInput,
    },
    primitive::Point,
};

mod reflow;

pub(super) struct RedrawScheduler {
    window: Arc<WinitWindow>,
    redraw_req_sent: Cell<bool>,
    reflow_nodes: RefCell<ReflowScheduler>,
    repaint_nodes: RefCell<HashMap<*const (), WeakHandle<dyn RenderTree>>>,
}

impl RedrawScheduler {
    pub fn new(window: Arc<WinitWindow>) -> Self {
        Self {
            window,
            redraw_req_sent: Cell::new(false),
            reflow_nodes: RefCell::new(ReflowScheduler::new()),
            repaint_nodes: Default::default(),
        }
    }

    fn request_window_redraw(&self) {
        if !self.redraw_req_sent.get() {
            self.redraw_req_sent.set(true);
            self.window.request_redraw();
        }
    }

    pub fn request_reflow(&self, this_el: &Element) {
        self.request_window_redraw();

        self.reflow_nodes.borrow_mut().push_reflow(this_el);
    }

    pub fn request_repaint(&self, el: &WeakElement) {
        self.request_window_redraw();

        self.repaint_nodes
            .borrow_mut()
            .insert(el.as_ptr() as _, el.clone());
    }

    pub fn redraw(
        &self,
        canvas: &Canvas,
        interval: Duration,
        root: &Element,
        redraw_root: Option<LayoutInput>,
    ) {
        let dirty_region = if let Some(redraw_root_inputs) = redraw_root {
            self.reflow_nodes.borrow_mut().clear();
            self.repaint_nodes.borrow_mut().clear();
            root.borrow_mut().compute_layout_cached(redraw_root_inputs);
            None
        } else {
            let dirty_region = self.perform_partial_reflow();
            canvas.save();
            canvas.clip_region(&dirty_region, ClipOp::Intersect);
            Some(dirty_region)
        };

        self.redraw_req_sent.set(false);

        canvas.clear(Color::WHITE);
        root.borrow_mut().render_entry(
            RenderArgs {
                canvas,
                interval,
                dirty_region: dirty_region.as_ref(),
            },
            Point::ZERO,
        );

        if dirty_region.is_some() {
            canvas.restore();
        }
    }

    fn perform_partial_reflow(&self) -> SkRegion {
        let mut dirty_region = SkRegion::new();

        for reflow_node in self.reflow_nodes.borrow_mut().get_reflow_roots() {
            let mut node = reflow_node.borrow_mut();
            let layout_input = node.common_mut().layout_input.expect(
                "this element has not been layouted before. its parent must reflow but it didn't.",
            );
            node.compute_layout_cached(layout_input);
        }

        for repaint_node in self
            .repaint_nodes
            .borrow_mut()
            .drain()
            .filter_map(|(_, node)| node.upgrade())
        {
            let mut node = repaint_node.borrow_mut();

            if let Some(prev_draw_region) = node.common_mut().prev_draw_region.take() {
                dirty_region.op_rect(prev_draw_region.round_to_skia_irect(), RegionOp::Union);
            }

            dirty_region.op_rect(
                node.common_mut()
                    .layout_output
                    .as_rect()
                    .round_to_skia_irect(),
                RegionOp::Union,
            );
        }

        dirty_region
    }
}
