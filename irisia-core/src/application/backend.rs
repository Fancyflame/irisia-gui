use std::{sync::Arc, time::Duration};

use irisia_backend::{
    skia_safe::Canvas,
    window_handle::{close_handle::CloseHandle, RawWindowHandle, WindowBuilder},
    AppWindow, StaticWindowEvent, WinitWindow,
};

use crate::{
    element::{render_content::BareContent, Element},
    event::EventDispatcher,
    primitive::Point,
    structure::{add_child, into_rendering_raw, node::AddChildCache, EmptyStructure},
    style::NoStyle,
    Result,
};

use super::{event_comp::EventComponent, Window};

pub(super) async fn new_window<El, F>(window_builder: F) -> Result<Window>
where
    El: Element<EmptyStructure>,
    F: FnOnce(WindowBuilder) -> WindowBuilder + Send + 'static,
{
    let ev_disp = EventDispatcher::new();

    let create_app = {
        let event_component = EventComponent::new(ev_disp.clone());
        move |window: Arc<WinitWindow>, close_handle| BackendRuntime::<El> {
            window,
            application: None,
            event_component,
            close_handle,
        }
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

pub(super) struct BackendRuntime<El: Element<EmptyStructure>> {
    window: Arc<WinitWindow>,
    application: Option<AddChildCache<El, El::Props>>,
    event_component: EventComponent,
    close_handle: CloseHandle,
}

impl<El> AppWindow for BackendRuntime<El>
where
    El: Element<EmptyStructure>,
{
    fn on_redraw(&mut self, canvas: &mut Canvas, size: (u32, u32), delta: Duration) -> Result<()> {
        let add_child =
            add_child::<El, _, _, _, _>(|_: &mut _| {}, NoStyle, |_| {}, EmptyStructure);

        let region = (Point(0.0, 0.0), Point(size.0 as _, size.1 as _));

        self.event_component
            .rebuild(|event_comp_builder, window_event_dispatcher, focusing| {
                let content = BareContent {
                    canvas,
                    window: &self.window,
                    delta_time: delta,
                    window_event_dispatcher,
                    close_handle: self.close_handle,
                    event_comp_builder,
                    focusing,
                };

                into_rendering_raw(add_child, &mut self.application, content).finish(region)
            })
    }

    fn on_window_event(&mut self, event: StaticWindowEvent) {
        self.event_component.emit_window_event(event);
    }
}
