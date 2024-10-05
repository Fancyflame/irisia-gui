use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
    sync::Arc,
    time::Duration,
};

use anyhow::anyhow;
use irisia_backend::{
    skia_safe::{colors::TRANSPARENT, Canvas},
    WinitWindow,
};
use smallvec::SmallVec;

use crate::Result;

pub(super) struct RedrawScheduler {
    window: Arc<WinitWindow>,
    list: RefCell<HashMap<*const dyn StandaloneRender, Rc<dyn StandaloneRender>>>,
    redraw_req_sent: Cell<bool>,
}

impl RedrawScheduler {
    pub fn new(window: Arc<WinitWindow>) -> Self {
        Self {
            window,
            list: Default::default(),
            redraw_req_sent: Cell::new(false),
        }
    }

    pub fn request_redraw(&self, ro: Rc<dyn StandaloneRender>) {
        if !self.redraw_req_sent.get() {
            self.redraw_req_sent.set(true);
            self.window.request_redraw();
        }
        self.list.borrow_mut().insert(Rc::as_ptr(&ro), ro);
    }

    pub fn redraw(&self, canvas: &Canvas, interval: Duration) -> Result<()> {
        let mut errors: SmallVec<[_; 2]> = SmallVec::new();

        loop {
            let mut list = self.list.borrow_mut();
            let ro = match list.keys().next() {
                Some(key) => {
                    let key = *key;
                    list.remove(&key).unwrap()
                }
                None => break,
            };
            drop(list);

            canvas.clear(TRANSPARENT);
            canvas.reset_matrix();

            if let Err(err) = ro.standalone_render(canvas, interval) {
                errors.push(err);
            }
        }

        self.redraw_req_sent.set(false);
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
        "{} error{} occurred on redraw: {}",
        errors.len(),
        if errors.len() > 1 { "s" } else { "" },
        msg
    ))
}
