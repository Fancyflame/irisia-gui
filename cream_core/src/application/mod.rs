use std::{sync::Arc, time::Duration};

use anyhow::anyhow;
use cream_backend::{
    skia_safe::Canvas,
    window_handle::{close_handle::CloseHandle, WindowBuilder, WindowHandle},
    AppWindow, WindowEvent, WinitWindow,
};
use tokio::{sync::mpsc, task::JoinHandle};

use crate::{
    element::{
        render_content::{RenderContent, WildRenderContent},
        Element,
    },
    event::{EventDispatcher, EventEmitter},
    primary::Point,
    structure::{
        add_child::{self, AddChildCache},
        EmptyStructure, Node,
    },
    style::NoStyle,
    Result,
};

use self::{elem_table::ElemTable, event::emit_to_elements};

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
    global_event_dispatcher: EventDispatcher,
    elem_table: ElemTable,
    event_sender: mpsc::UnboundedSender<WindowEvent>,
    event_sender_handle: JoinHandle<()>,
    close_handle: CloseHandle,
    cursor_pos: Point,
}

impl<El> AppWindow for Application<El>
where
    El: Element,
{
    fn on_create(window: &std::sync::Arc<WinitWindow>, close_handle: CloseHandle) -> Result<Self> {
        let dispatcher = EventDispatcher::new();
        let (tx, mut rx) = mpsc::unbounded_channel();

        let event_sender_handle = {
            let dispatcher = dispatcher.clone();
            tokio::spawn(async move {
                let emitter = dispatcher.get_emitter().await;
                loop {
                    let event = match rx.recv().await {
                        Some(event) => event,
                        None => return,
                    };

                    emitter.emit(&event).await;
                }
            })
        };

        Ok(Application {
            window: window.clone(),
            application: None,
            global_event_dispatcher: dispatcher,
            elem_table: ElemTable::new(),
            event_sender: tx,
            event_sender_handle,
            close_handle,
            cursor_pos: Default::default(),
        })
    }

    fn on_redraw(&mut self, canvas: &mut Canvas, size: (u32, u32), delta: Duration) -> Result<()> {
        let add_child = add_child::add_child::<El, _, _>(
            Default::default(),
            NoStyle,
            EventEmitter::new_no_receiver(),
            EmptyStructure,
        );

        let mut region = Some((Point(0, 0), Point(size.0, size.1)));

        let content = WildRenderContent(RenderContent {
            canvas,
            window: &self.window,
            delta_time: delta,
            global_event_receiver: &self.global_event_dispatcher,
            close_handle: self.close_handle,
            elem_table_index: None,
            elem_table_builder: self.elem_table.builder(),
        });

        add_child.__finish_iter(&mut self.application, content, &mut move |_: (), _| {
            region.take().ok_or_else(|| {
                anyhow!("at most one element is allowed to be rendered as root of the window")
            })
        })
    }

    fn on_window_event(&mut self, event: WindowEvent) {
        self.event_sender
            .send(event.clone())
            .expect("inner error: global window event sender dumped");

        emit_to_elements(&mut self.elem_table, &mut self.cursor_pos, event);
    }
}

impl<El> Drop for Application<El> {
    fn drop(&mut self) {
        self.event_sender_handle.abort();
    }
}
