use std::sync::Weak;

use irisia_backend::{winit::window::WindowAttributes, WinitWindow};

use crate::{
    event::{standard::WindowDestroyed, EventDispatcher},
    model::{iter::basic::ModelBasicMapper, RootDesiredModel},
    Result,
};

mod backend;
pub(crate) mod content;
pub(crate) mod event_comp;
pub(crate) mod redraw_scheduler;

use backend::new_window;

pub use event_comp::IncomingPointerEvent;
pub use irisia_backend::window_handle::CloseHandle;

#[derive(Clone)]
pub struct Window {
    winit_window: Weak<WinitWindow>,
    close_handle: CloseHandle,
    event_dispatcher: EventDispatcher,
}

impl Window {
    pub async fn new<T>(wa: WindowAttributes, dom: T) -> Result<Self>
    where
        T: RootDesiredModel<ModelBasicMapper, RootCp = ()> + Send + 'static,
    {
        new_window(wa, dom).await
    }

    pub fn winit_window(&self) -> &Weak<WinitWindow> {
        &self.winit_window
    }

    pub fn close_handle(&self) -> CloseHandle {
        self.close_handle
    }

    pub fn close(&self) {
        self.close_handle.close();
    }

    pub fn event_dispatcher(&self) -> &EventDispatcher {
        &self.event_dispatcher
    }

    pub async fn join(&self) {
        if self.winit_window.strong_count() == 0 {
            return;
        }

        self.event_dispatcher
            .recv_trusted::<WindowDestroyed>()
            .await;
    }
}
