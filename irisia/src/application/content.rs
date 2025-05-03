use std::{cell::Cell, sync::Arc};

use irisia_backend::{WinitWindow, window_handle::CloseHandle};

use crate::{
    event::EventDispatcher,
    primitive::{Region, length::LengthStandardGlobalPart},
};

use super::{event_comp::global::focusing::Focusing, redraw_scheduler::RedrawScheduler};

pub struct GlobalContent {
    pub(super) focusing: Focusing,
    pub(super) global_ed: EventDispatcher,
    pub(super) window: Arc<WinitWindow>,
    pub(super) length_standard: Cell<LengthStandardGlobalPart>,
    pub(super) close_handle: CloseHandle,
    pub(super) user_close: Cell<bool>,
    pub(super) redraw_scheduler: RedrawScheduler,
}

impl GlobalContent {
    pub fn blur(&self) {
        self.focusing.blur();
    }

    pub(crate) fn focusing(&self) -> &Focusing {
        &self.focusing
    }

    pub(crate) fn request_redraw(&self, dirty_region: Region) {
        self.redraw_scheduler.request_redraw(dirty_region)
    }

    pub(crate) fn request_relayout(&self) {
        self.redraw_scheduler.request_relayout();
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

    pub fn length_standard(&self) -> LengthStandardGlobalPart {
        self.length_standard.get()
    }

    pub fn user_close(&self) -> bool {
        self.user_close.get()
    }

    pub fn set_user_close(&self, enable: bool) {
        self.user_close.set(enable);
    }
}
