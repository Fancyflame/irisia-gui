use std::time::Duration;

use cream_backend::{skia_safe::Canvas, window::Window};

use crate::{event::EventReceiver, primary::Region};

pub struct RenderContent<'a> {
    pub(crate) canvas: &'a mut Canvas,
    pub(crate) region: Region,
    pub(crate) window: &'a Window,
    pub(crate) delta: Duration,
    pub(crate) global_event_receiver: &'a EventReceiver,
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

    pub fn window(&self) -> &Window {
        self.window
    }

    pub fn delta_time(&self) -> Duration {
        self.delta
    }

    pub fn inherit(&mut self, mut new_region: Region) -> RenderContent<'_> {
        #[cfg(debug_assertions)]
        if !(new_region.0.abs_gt(self.region.0) && new_region.1.abs_lt(self.region.1)) {
            panic!("new region must be contained in current region");
        } else {
            new_region = self.region;
        }

        RenderContent {
            region: new_region,
            canvas: self.canvas,
            window: self.window,
            delta: self.delta,
            global_event_receiver: self.global_event_receiver,
        }
    }
}
