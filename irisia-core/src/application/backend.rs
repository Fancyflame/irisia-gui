use std::{sync::Arc, time::Duration};

use irisia_backend::{
    skia_safe::{colors::TRANSPARENT, Canvas},
    window_handle::{close_handle::CloseHandle, RawWindowHandle, WindowBuilder},
    AppWindow, StaticWindowEvent, WinitWindow,
};

use crate::{
    element::{render_content::BareContent, Element, ElementMutate},
    event::EventDispatcher,
    primitive::Point,
    structure::{
        add_child, into_rendering_raw,
        layer::{LayerCompositer, SharedLayerCompositer},
        node::AddChildCache,
        EmptyStructure,
    },
    style::NoStyle,
    Result,
};

use super::{event_comp::EventComponent, Window};

pub(super) async fn new_window<El, F>(window_builder: F) -> Result<Window>
where
    El: Element + ElementMutate<(), EmptyStructure>,
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
            layer_compositer: LayerCompositer::new_shared(),
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

pub(super) struct BackendRuntime<El> {
    window: Arc<WinitWindow>,
    application: Option<AddChildCache<El>>,
    event_component: EventComponent,
    close_handle: CloseHandle,
    layer_compositer: SharedLayerCompositer,
}

impl<El> AppWindow for BackendRuntime<El>
where
    El: Element + ElementMutate<(), EmptyStructure>,
{
    fn on_redraw(&mut self, canvas: &mut Canvas, size: (u32, u32), delta: Duration) -> Result<()> {
        let add_child = add_child::<El, _, _, _, _>((), NoStyle, EmptyStructure, |_: &_| {});

        let region = (Point(0.0, 0.0), Point(size.0 as _, size.1 as _));

        self.event_component
            .rebuild(|event_comp_builder, window_event_dispatcher, focusing| {
                let content = BareContent {
                    window: &self.window,
                    delta_time: delta,
                    window_event_dispatcher,
                    close_handle: self.close_handle,
                    event_comp_builder,
                    focusing,
                };

                let mut lc = self.layer_compositer.borrow_mut();

                if self.application.is_none() {
                    into_rendering_raw(add_child, &mut self.application, content)
                        .finish(region, true)?;
                }

                canvas.clear(TRANSPARENT);
                lc.composite(canvas)
            })
    }

    fn on_window_event(&mut self, event: StaticWindowEvent) {
        self.event_component.emit_window_event(event);
    }
}
