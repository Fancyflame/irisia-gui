use std::{sync::Arc, time::Duration};

use cream_backend::{
    skia_safe::Canvas,
    window_handle::{close_handle::CloseHandle, WindowBuilder, WindowHandle},
    AppWindow, WindowEvent, WinitWindow,
};
use tokio::{sync::mpsc, task::JoinHandle};

use crate::{
    element::{Element, RenderContent},
    event::{event_channel::one_channel, EventEmitter, EventReceiver},
    primary::Point,
    structure::{
        add_child::{self, AddChildCache},
        EmptyStructure, Node,
    },
    style::NoStyle,
    Result,
};

pub mod event;

pub async fn new_window<El, F>(window_builder: F) -> Result<WindowHandle<Application<El>>>
where
    El: Element<Children<EmptyStructure> = EmptyStructure>,
    F: FnOnce(WindowBuilder) -> WindowBuilder + Send + 'static,
{
    WindowHandle::create(window_builder).await
}

pub struct Application<El> {
    window: Arc<WinitWindow>,
    application: Option<AddChildCache<El, ()>>,
    global_event_receiver: EventReceiver,
    event_sender: mpsc::UnboundedSender<WindowEvent>,
    event_sender_handle: JoinHandle<()>,
    close_handle: CloseHandle,
}

impl<El> AppWindow for Application<El>
where
    El: Element<Children<EmptyStructure> = EmptyStructure>,
{
    fn on_create(window: &std::sync::Arc<WinitWindow>, close_handle: CloseHandle) -> Result<Self> {
        let (emitter, receiver) = one_channel();
        let (tx, mut rx) = mpsc::unbounded_channel();

        let event_sender_handle = tokio::spawn(async move {
            loop {
                let event = match rx.recv().await {
                    Some(event) => event,
                    None => return,
                };

                emitter.emit(event).await;
            }
        });

        Ok(Application {
            window: window.clone(),
            application: None,
            global_event_receiver: receiver,
            event_sender: tx,
            event_sender_handle,
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

        add_child.finish_iter(
            &mut self.application,
            std::iter::once(RenderContent {
                canvas,
                region: (Point(0, size.0), Point(0, size.1)),
                window: &self.window,
                delta,
                global_event_receiver: &self.global_event_receiver,
                close_handle: self.close_handle.clone(),
            }),
        )
    }

    fn on_window_event(&mut self, event: WindowEvent) {
        self.event_sender
            .send(event)
            .expect("inner error: global window event sender dumped");
    }
}

impl<El> Drop for Application<El> {
    fn drop(&mut self) {
        self.event_sender_handle.abort();
    }
}
