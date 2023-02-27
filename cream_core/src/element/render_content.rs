use skia_safe::Canvas;

use crate::{
    event::{
        event_state::wrap::WrappedEvents,
        global_register::{system_event_register::SystemEventRegister, GlobalEventRegister},
    },
    primary::Region,
};

pub struct RenderContent<'a> {
    pub(crate) canvas: &'a mut Canvas,
    pub(crate) event_register: &'a mut SystemEventRegister,
    pub(crate) region: Region,
}

impl RenderContent<'_> {
    pub fn canvas_ref(&self) -> &Canvas {
        &self.canvas
    }

    pub fn canvas(&mut self) -> &mut Canvas {
        &mut self.canvas
    }

    pub fn region(&self) -> Region {
        self.region
    }

    pub fn listen_global<E, F>(&mut self, w: WrappedEvents) {
        self.event_register.listen_list(w, self.region);
    }

    pub fn inherit(&mut self, new_region: Region) -> RenderContent<'_> {
        #[cfg(debug_assertions)]
        if !(new_region.0.abs_gt(self.region.0) && new_region.1.abs_lt(self.region.1)) {
            panic!("new region must be contained in current region");
        }

        RenderContent {
            canvas: &mut self.canvas,
            event_register: &mut *self.event_register,
            region: new_region,
        }
    }
}
