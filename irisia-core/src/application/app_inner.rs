use std::{sync::Arc, time::Duration};

use irisia_backend::{
    skia_safe::Canvas,
    window_handle::{close_handle::CloseHandle, RawWindowHandle, WindowBuilder},
    AppWindow, StaticWindowEvent, WinitWindow,
};

use crate::{
    element::{render_content::BareContent, Element},
    event::{event_dispatcher::emitter::CreatedEventEmitter, EventDispatcher},
    primary::Point,
    structure::{
        add_child::{self, AddChildCache},
        into_rendering_raw, EmptyStructure,
    },
    style::NoStyle,
    Result,
};

use super::{elem_table::ElemTable, Window};

pub(super) async fn new_window<El, F>(window_builder: F) -> Result<Window>
where
    El: Element,
    F: FnOnce(WindowBuilder) -> WindowBuilder + Send + 'static,
{
    let ev_disp = EventDispatcher::new();

    let ev_disp_cloned = ev_disp.clone();
    let create_app = move |window: Arc<WinitWindow>, close_handle: CloseHandle| Application::<El> {
        window,
        application: None,
        elem_table: ElemTable::new(ev_disp_cloned),
        close_handle,
    };

    let RawWindowHandle {
        raw_window,
        close_handle,
    } = RawWindowHandle::create(create_app, window_builder).await?;

    Ok(Window {
        winit_window: raw_window,
        close_handle,
        event_dispatcher: ev_disp,
    })
}

pub(super) struct Application<El> {
    window: Arc<WinitWindow>,
    application: Option<AddChildCache<El, ()>>,
    elem_table: ElemTable,
    close_handle: CloseHandle,
}

impl<El> AppWindow for Application<El>
where
    El: Element,
{
    fn on_redraw(&mut self, canvas: &mut Canvas, size: (u32, u32), delta: Duration) -> Result<()> {
        let add_child = add_child::add_child::<El, _, (), _>(
            Default::default(),
            NoStyle,
            CreatedEventEmitter::new_empty(),
            EmptyStructure,
        );

        let region = (Point(0, 0), Point(size.0, size.1));

        let (elem_table_builder, window_event_receiver) = self.elem_table.builder();
        let content = BareContent {
            canvas,
            window: &self.window,
            delta_time: delta,
            window_event_receiver,
            close_handle: self.close_handle,
            elem_table_builder,
        };

        into_rendering_raw(add_child, &mut self.application, content).finish(region)
    }

    fn on_window_event(&mut self, event: StaticWindowEvent) {
        self.elem_table.emit_window_event(event);
    }
}
