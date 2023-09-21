use std::{collections::HashMap, rc::Rc, sync::Arc, time::Duration};

use anyhow::anyhow;
use irisia_backend::{
    skia_safe::{colors::TRANSPARENT, Canvas},
    WinitWindow,
};

use crate::Result;

pub(super) struct RedrawScheduler {
    window: Arc<WinitWindow>,
    list: HashMap<*const dyn RedrawObject, Rc<dyn RedrawObject>>,
    redraw_req_sent: bool,
}

impl RedrawScheduler {
    pub fn new(window: Arc<WinitWindow>) -> Self {
        Self {
            window,
            list: HashMap::new(),
            redraw_req_sent: false,
        }
    }

    pub fn request_redraw(&mut self, ro: Rc<dyn RedrawObject>) {
        if !self.redraw_req_sent {
            self.redraw_req_sent = true;
            self.window.request_redraw();
        }
        self.list.insert(Rc::as_ptr(&ro), ro);
    }

    pub fn redraw(&mut self, canvas: &mut Canvas, interval: Duration) -> Result<()> {
        let mut errors = Vec::new();
        self.redraw_req_sent = false;

        for (_, ro) in self.list.drain() {
            canvas.clear(TRANSPARENT);
            canvas.reset_matrix();

            if let Err(err) = ro.redraw(canvas, interval) {
                errors.push(err);
            }
        }

        fmt_errors(&errors)
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
        "{} error(s) occurred on redraw: {}",
        errors.len(),
        msg
    ))
}

pub(crate) trait RedrawObject {
    fn redraw(&self, canvas: &mut Canvas, interval: Duration) -> Result<()>;
}
