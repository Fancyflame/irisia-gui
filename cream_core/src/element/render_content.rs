use std::time::Duration;

use cream_backend::{window::Window, Canvas};

use crate::{
    event::{event_state::wrap::WrappedEvents, global_event_register::SystemEventRegister},
    primary::Region,
};

pub struct RenderContent<'a> {
    pub(crate) canvas: &'a mut Canvas,
    pub(crate) event_register: &'a mut SystemEventRegister,
    pub(crate) region: Region,
    pub(crate) window: &'a Window,
    pub(crate) delta: Duration,
}

impl RenderContent<'_> {
    pub fn canvas_ref(&self) -> &Canvas {
        self.canvas
    }

    pub fn canvas(&mut self) -> &mut Canvas {
        self.canvas
    }

    pub fn region(&self) -> Region {
        self.region
    }

    pub fn listen_global<E, F>(&mut self, w: WrappedEvents) {
        self.event_register.listen_list(w, self.region);
    }

    pub fn window(&self) -> &Window {
        self.window
    }

    pub fn delta_time(&self) -> Duration {
        self.delta
    }

    pub fn inherit(&mut self, new_region: Region) -> RenderContent<'_> {
        #[cfg(debug_assertions)]
        if !(new_region.0.abs_gt(self.region.0) && new_region.1.abs_lt(self.region.1)) {
            panic!("new region must be contained in current region");
        }

        RenderContent {
            canvas: self.canvas,
            event_register: self.event_register,
            region: new_region,
            window: self.window,
            delta: self.delta,
        }
    }
}
