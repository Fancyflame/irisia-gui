use std::{cell::RefCell, rc::Rc, sync::Arc};

use irisia_backend::{window_handle::CloseHandle, WinitWindow};

use crate::event::EventDispatcher;

use super::{
    event_comp::global::focusing::Focusing,
    redraw_scheduler::{RedrawObject, RedrawScheduler},
};

pub struct GlobalContent {
    pub(super) focusing: Focusing,
    pub(super) global_ed: EventDispatcher,
    pub(super) window: Arc<WinitWindow>,
    pub(super) close_handle: CloseHandle,
    pub(super) redraw_scheduler: RefCell<RedrawScheduler>,
}

impl GlobalContent {
    pub fn blur(&self) {
        self.focusing.blur();
    }

    pub(crate) fn focusing(&self) -> &Focusing {
        &self.focusing
    }

    pub(crate) fn request_redraw(&self, ro: Rc<dyn RedrawObject>) {
        self.redraw_scheduler.borrow_mut().request_redraw(ro)
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
}
