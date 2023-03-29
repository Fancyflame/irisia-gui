use std::{sync::Arc, time::Duration};

use cream_backend::{
    skia_safe::Canvas,
    window_handle::{close_handle::CloseHandle, WindowBuilder, WindowHandle},
    winit::dpi::PhysicalPosition,
    AppWindow, WindowEvent, WinitWindow,
};

use crate::{
    element::{
        render_content::{RenderContent, WildRenderContent},
        Element,
    },
    event::{EventDispatcher, EventEmitter},
    primary::Point,
    structure::{
        add_child::{self, AddChildCache},
        into_rendering_raw, EmptyStructure,
    },
    style::NoStyle,
    Result,
};

use self::elem_table::ElemTable;

pub(crate) mod elem_table;
pub mod event;

pub async fn new_window<El, F>(window_builder: F) -> Result<WindowHandle<Application<El>>>
where
    El: Element,
    F: FnOnce(WindowBuilder) -> WindowBuilder + Send + 'static,
{
    WindowHandle::create(window_builder).await
}

pub struct Application<El> {
    window: Arc<WinitWindow>,
    application: Option<AddChildCache<El, ()>>,
    window_event_dispatcher: EventDispatcher,
    elem_table: ElemTable,
    close_handle: CloseHandle,
}

impl<El> AppWindow for Application<El>
where
    El: Element,
{
    fn on_create(window: &std::sync::Arc<WinitWindow>, close_handle: CloseHandle) -> Result<Self> {
        Ok(Application {
            window: window.clone(),
            application: None,
            window_event_dispatcher: EventDispatcher::new(),
            elem_table: ElemTable::new(),
            close_handle,
        })
    }

    fn on_redraw(&mut self, canvas: &mut Canvas, size: (u32, u32), delta: Duration) -> Result<()> {
        let add_child = add_child::add_child::<El, _, _>(
            Default::default(),
            NoStyle,
            EventEmitter::new_empty(),
            EmptyStructure,
        );

        let region = (Point(0, 0), Point(size.0, size.1));

        let content = WildRenderContent(RenderContent {
            canvas,
            window: &self.window,
            delta_time: delta,
            window_event_receiver: &self.window_event_dispatcher,
            close_handle: self.close_handle,
            elem_table_index: None,
            elem_table_builder: self.elem_table.builder(),
        });

        into_rendering_raw(add_child, &mut self.application, content).finish(region)
    }

    fn on_window_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::CursorMoved {
                position: PhysicalPosition { x, y },
                ..
            } => self.elem_table.update_cursor(Some(Point(x as _, y as _))),
            //WindowEvent::CursorLeft { .. } => self.elem_table.update_cursor(None),
            _ => {}
        }

        self.window_event_dispatcher.emit(&event, &());
        self.elem_table.emit(&event);
    }
}
