use std::sync::{atomic::AtomicBool, Arc};

use irisia_backend::{window_handle::CloseHandle, WinitWindow};

use crate::event::EventDispatcher;

use super::event_comp::global::focusing::Focusing;

pub struct GlobalContent {
    pub(super) focusing: Focusing,
    pub(super) global_ed: EventDispatcher,
    pub(super) window: Arc<WinitWindow>,
    pub(super) close_handle: CloseHandle,
    pub(super) is_dirty: AtomicBool,
}

impl GlobalContent {
    pub(crate) fn set_dirty(&self) {
        self.is_dirty
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.window.request_redraw();
    }

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

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }
}
