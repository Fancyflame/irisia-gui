use std::sync::Arc;

use irisia_backend::{window_handle::CloseHandle, WinitWindow};

use crate::event::EventDispatcher;

use super::event_comp::global::focusing::Focusing;

pub struct GlobalContent {
    pub(super) focusing: Focusing,
    pub(super) global_ed: EventDispatcher,
    pub(super) window: Arc<WinitWindow>,
    pub(super) close_handle: CloseHandle,
}

impl GlobalContent {
    pub fn blur(&self) {
        self.focusing.blur();
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

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }
}
