use std::sync::{Arc, Mutex as StdMutex};

use irisia_backend::{window_handle::CloseHandle, WinitWindow};

use crate::event::EventDispatcher;

use super::{
    event_comp::global::focusing::Focusing,
    redraw_scheduler::{list::RedrawList, LayerId},
};

pub struct GlobalContent {
    pub(super) focusing: Focusing,
    pub(super) global_ed: EventDispatcher,
    pub(super) window: Arc<WinitWindow>,
    pub(super) close_handle: CloseHandle,
    pub(super) redraw_list: StdMutex<RedrawList>,
}

impl GlobalContent {
    pub fn blur(&self) {
        self.focusing.blur();
    }

    pub(crate) fn focusing(&self) -> &Focusing {
        &self.focusing
    }

    pub fn global_event_dispatcher(&self) -> &EventDispatcher {
        &self.global_ed
    }

    pub fn close_handle(&self) -> CloseHandle {
        self.close_handle
    }

    pub fn close_window(&self) {
        self.close_handle.close();
    }

    pub fn window(&self) -> &WinitWindow {
        &self.window
    }

    pub(crate) fn request_redraw(&self, id: LayerId) {
        self.redraw_list.lock().unwrap().request_redraw(id);
    }
}
