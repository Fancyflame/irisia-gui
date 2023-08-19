use std::sync::Arc;

use irisia_backend::WinitWindow;

use super::LayerId;

pub struct RedrawList {
    pub(super) window: Arc<WinitWindow>,
    pub(super) list: Vec<LayerId>,
    pub(super) redraw_req_sent: bool,
}

impl RedrawList {
    pub fn request_redraw(&mut self, id: LayerId) {
        if !self.redraw_req_sent {
            self.redraw_req_sent = true;
            self.window.request_redraw();
        }
        self.list.push(id);
    }
}
