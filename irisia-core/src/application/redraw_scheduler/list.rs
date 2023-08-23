use std::{
    collections::{hash_set::Drain, HashSet},
    sync::Arc,
};

use irisia_backend::WinitWindow;

use super::LayerId;

pub(crate) struct RedrawList {
    window: Arc<WinitWindow>,
    list: HashSet<LayerId>,
    redraw_req_sent: bool,
}

impl RedrawList {
    pub(super) fn new(window: Arc<WinitWindow>) -> Self {
        Self {
            window,
            list: HashSet::new(),
            redraw_req_sent: false,
        }
    }

    pub fn request_redraw(&mut self, id: LayerId) {
        if !self.redraw_req_sent {
            self.redraw_req_sent = true;
            self.window.request_redraw();
        }
        self.list.insert(id);
    }

    pub fn drain(&mut self) -> Drain<LayerId> {
        self.redraw_req_sent = false;
        self.list.drain()
    }
}
