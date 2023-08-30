use std::sync::{Arc, Mutex as StdMutex};

use irisia_backend::{window_handle::CloseHandle, WinitWindow};

use crate::event::EventDispatcher;

use super::{
    event_comp::global::focusing::Focusing,
    redraw_scheduler::{LayerId, RedrawList},
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

    /// Returns a reference to the global event dispatcher
    pub fn event_dispatcher(&self) -> &EventDispatcher {
        &self.global_ed
    }

    /// Returns the close handle
    pub fn close_handle(&self) -> CloseHandle {
        self.close_handle
    }

    /// Closes the window
    pub fn close_window(&self) {
        self.close_handle.close();
    }

    /// Returns a reference to the window
    pub fn window(&self) -> &WinitWindow {
        &self.window
    }

    pub(crate) fn request_redraw(&self, id: LayerId) {
        self.redraw_list.lock().unwrap().request_redraw(id);
    }
}
