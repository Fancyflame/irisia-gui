use std::sync::Arc;

use cream_backend::{
    window_handle::{close_handle::CloseHandle, WindowBuilder},
    WinitWindow,
};

use crate::{
    element::Element,
    event::{standard::window_event::WindowDestroyed, EventDispatcher},
    Result,
};

mod app_inner;
pub(crate) mod elem_table;

use app_inner::new_window;

#[derive(Clone)]
pub struct Window {
    winit_window: Arc<WinitWindow>,
    close_handle: CloseHandle,
    event_dispatcher: EventDispatcher,
}

impl Window {
    pub async fn new<El: Element>(title: String) -> Result<Self> {
        new_window::<El, _>(move |wb| wb.with_title(title)).await
    }

    pub async fn with_builder<El, F>(f: F) -> Result<Self>
    where
        El: Element,
        F: FnOnce(WindowBuilder) -> WindowBuilder + Send + 'static,
    {
        new_window::<El, _>(f).await
    }

    pub fn winit_window(&self) -> &Arc<WinitWindow> {
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
        self.event_dispatcher.recv_sys::<WindowDestroyed>().await;
    }
}
