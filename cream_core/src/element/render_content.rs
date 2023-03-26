use std::time::Duration;

use anyhow::anyhow;
use cream_backend::{skia_safe::Canvas, window_handle::close_handle::CloseHandle, WinitWindow};

use crate::{application::elem_table, event::EventDispatcher, primary::Region, Result};

pub struct RenderContent<'a> {
    pub(crate) canvas: &'a mut Canvas,
    pub(crate) window: &'a WinitWindow,
    pub(crate) delta_time: Duration,
    pub(crate) global_event_receiver: &'a EventDispatcher,
    pub(crate) close_handle: CloseHandle,
    pub(crate) elem_table_index: Option<usize>,
    pub(crate) elem_table_builder: elem_table::Builder<'a>,
    pub(crate) region_requested: bool,
    pub(crate) region_requester: Option<&'a mut dyn FnMut(Option<Region>) -> Result<Region>>,
}

pub struct WildRenderContent<'a>(pub(crate) RenderContent<'a>);

impl RenderContent<'_> {
    pub fn canvas_ref(&self) -> &Canvas {
        self.canvas
    }

    pub fn canvas(&mut self) -> &mut Canvas {
        self.canvas
    }

    pub fn window(&self) -> &WinitWindow {
        self.window
    }

    pub fn delta_time(&self) -> Duration {
        self.delta_time
    }

    pub fn set_interact_region(&mut self, region: Region) {
        self.elem_table_builder.set_interact_region_for(
            self.elem_table_index.expect(
                "inner error: cannot set interact region, beacause the element not rendered",
            ),
            region,
        );
    }

    pub fn request_drawing_region(&mut self, sized: Option<Region>) -> Result<Region> {
        if self.region_requested {
            Err(anyhow!("one element can only request region once"))
        } else {
            self.region_requested = true;
            match &mut self.region_requester {
                Some(req) => (req)(sized),
                None => panic!("inner error: region requester not setted"),
            }
        }
    }

    pub(crate) fn downgrade_lifetime(&mut self) -> RenderContent {
        RenderContent {
            canvas: self.canvas,
            window: self.window,
            delta_time: self.delta_time,
            global_event_receiver: self.global_event_receiver,
            close_handle: self.close_handle,
            elem_table_index: self.elem_table_index,
            elem_table_builder: self.elem_table_builder.downgrade_lifetime(),
            region_requester: match self.region_requester {
                Some(ref mut f) => Some(*f),
                None => None,
            },
            region_requested: self.region_requested,
        }
    }

    pub fn inherit(&mut self) -> WildRenderContent<'_> {
        let mut content = self.downgrade_lifetime();
        content.elem_table_index = None;
        content.region_requested = false;
        content.region_requester = None;
        WildRenderContent(content)
    }
}

impl<'a> WildRenderContent<'a> {
    pub(crate) fn downgrade_lifetime(&mut self) -> WildRenderContent {
        WildRenderContent(self.0.downgrade_lifetime())
    }
}
