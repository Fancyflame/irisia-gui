use std::{
    cell::{Cell, RefCell},
    sync::Arc,
    time::Duration,
};

use anyhow::anyhow;
use irisia_backend::{
    skia_safe::{region::RegionOp, Canvas, ClipOp, Region as SkRegion},
    WinitWindow,
};

use crate::{
    el_model::{ElementAccess, ElementModel},
    element::Render,
    ElementInterfaces, Result,
};

pub(super) struct RedrawScheduler {
    window: Arc<WinitWindow>,
    dirty_regions: RefCell<Vec<ElementAccess>>,
    redrawing: RefCell<SkRegion>,
    redraw_req_sent: Cell<bool>,
}

impl RedrawScheduler {
    pub fn new(window: Arc<WinitWindow>) -> Self {
        Self {
            window,
            dirty_regions: RefCell::new(Vec::new()),
            redrawing: RefCell::new(SkRegion::new()),
            redraw_req_sent: Cell::new(false),
        }
    }

    pub fn request_redraw(&self, access: ElementAccess) {
        if !self.redraw_req_sent.get() {
            self.redraw_req_sent.set(true);
            self.window.request_redraw();
        }
        self.dirty_regions.borrow_mut().push(access);
    }

    pub fn redraw<Root>(
        &self,
        root: &mut ElementModel<Root, ()>,
        canvas: &Canvas,
        interval: Duration,
    ) -> Result<()>
    where
        Root: ElementInterfaces,
    {
        let mut dirty_region = self.redrawing.borrow_mut();
        let mut unmerged = self.dirty_regions.borrow_mut();
        dirty_region.set_empty();
        for access in unmerged.drain(..) {
            let (old_region, new_region) = access.reset_redraw_region_pair();
            if let Some(old) = old_region {
                dirty_region.op_rect(old.ceil_to_irect(), RegionOp::Union);
            }
            dirty_region.op_rect(new_region.ceil_to_irect(), RegionOp::Union);
        }
        drop(unmerged);
        if dirty_region.is_empty() {
            return Ok(());
        }

        self.redraw_req_sent.set(false);
        canvas.save();
        canvas.clip_region(&dirty_region, ClipOp::Max_EnumValue);
        let res = root.render(Render {
            canvas,
            interval,
            dirty_region: &dirty_region,
        });
        canvas.restore();
        res
    }
}

fn fmt_errors(errors: &[anyhow::Error]) -> Result<()> {
    if errors.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    for (index, err) in errors.iter().enumerate() {
        msg += &format!("`{err}`");
        if index != errors.len() - 1 {
            msg += ", ";
        }
    }

    Err(anyhow!(
        "{} error{} occurred on redraw: {}",
        errors.len(),
        if errors.len() > 1 { "s" } else { "" },
        msg
    ))
}
