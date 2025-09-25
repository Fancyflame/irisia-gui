use std::sync::Weak;

use irisia_backend::{
    WinitWindow,
    winit::{dpi::PhysicalSize, window::WindowAttributes},
};

use crate::{
    Result,
    event::{EventDispatcher, standard::WindowDestroyed},
    model::VNode,
    prim_element::layout::SpaceConstraint,
    primitive::size::Size,
};

mod backend;
pub(crate) mod content;
pub(crate) mod event2;
pub(crate) mod event_comp;
pub(crate) mod redraw_scheduler;

use backend::new_window;

pub use irisia_backend::window_handle::CloseHandle;
pub use {event_comp::IncomingPointerEvent, event2::pointer_event::PointerEvent};

#[derive(Clone)]
pub struct Window {
    winit_window: Weak<WinitWindow>,
    close_handle: CloseHandle,
    event_dispatcher: EventDispatcher,
}

impl Window {
    pub async fn new<F, T>(wa: WindowAttributes, dom: F) -> Result<Self>
    where
        F: FnOnce() -> T + Send + 'static,
        T: VNode,
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

fn window_size_to_constraint(size: PhysicalSize<u32>) -> Size<SpaceConstraint> {
    Size {
        width: SpaceConstraint::Exact(size.width as f32),
        height: SpaceConstraint::Exact(size.height as f32),
    }
}
