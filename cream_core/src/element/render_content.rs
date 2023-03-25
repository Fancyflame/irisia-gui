use std::time::Duration;

use cream_backend::{skia_safe::Canvas, window_handle::close_handle::CloseHandle, WinitWindow};

use crate::{application::elem_table, event::EventDispatcher, primary::Region};

pub struct RenderContent<'a> {
    pub(crate) canvas: &'a mut Canvas,
    pub(crate) window: &'a WinitWindow,
    pub(crate) delta_time: Duration,
    pub(crate) global_event_receiver: &'a EventDispatcher,
    pub(crate) close_handle: CloseHandle,
    pub(crate) elem_table_index: Option<usize>,
    pub(crate) elem_table_builder: elem_table::Builder<'a>,
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

    pub(crate) fn downgrade_lifetime(&mut self) -> RenderContent {
        RenderContent {
            canvas: self.canvas,
            window: self.window,
            delta_time: self.delta_time,
            global_event_receiver: self.global_event_receiver,
            close_handle: self.close_handle,
            elem_table_index: self.elem_table_index,
            elem_table_builder: self.elem_table_builder.downgrade_lifetime(),
        }
    }

    pub fn inherit(&mut self) -> WildRenderContent<'_> {
        let mut content = self.downgrade_lifetime();
        content.elem_table_index = None;
        WildRenderContent(content)
    }
}

impl<'a> WildRenderContent<'a> {
    pub(crate) fn downgrade_lifetime(&mut self) -> WildRenderContent {
        WildRenderContent(self.0.downgrade_lifetime())
    }
}
