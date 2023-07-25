use std::{
    sync::{Arc, Weak},
    time::Duration,
};

use irisia_backend::{window_handle::CloseHandle, WinitWindow};
use tokio::sync::Mutex;

use crate::{
    element::{EventHandle, InitContent},
    event::EventDispatcher,
};

use super::event_comp::global::focusing::Focusing;

pub(crate) struct GlobalContent<'a> {
    pub global_ed: &'a EventDispatcher,
    pub focusing: &'a Focusing,
    pub window: &'a Arc<WinitWindow>,
    pub close_handle: CloseHandle,
    pub interval: Duration,
}

impl GlobalContent<'_> {
    pub fn build_init_content<T>(&self, app: Weak<Mutex<T>>) -> InitContent<T> {
        InitContent {
            app,
            event_handle: EventHandle::new(EventDispatcher::new(), self.focusing.clone()),
            window_event_dispatcher: self.global_ed.clone(),
            window: self.window.clone(),
            close_handle: self.close_handle,
        }
    }

    pub fn downgrade_lifetime(&self) -> GlobalContent {
        GlobalContent {
            global_ed: self.global_ed,
            focusing: self.focusing,
            window: self.window,
            close_handle: self.close_handle,
            interval: self.interval,
        }
    }
}
