use std::{sync::Arc, time::Duration};

use irisia_backend::{
    skia_safe::{colors::TRANSPARENT, Canvas},
    window_handle::{close_handle::CloseHandle, RawWindowHandle, WindowBuilder},
    AppWindow, StaticWindowEvent, WinitWindow,
};

use crate::{
    element::{ChildrenCache, Element, ElementMutate},
    event::EventDispatcher,
    primitive::{Pixel, Point},
    structure::{
        add_child,
        layer::{LayerCompositer, SharedLayerCompositer},
        node::AddChildCache,
        TreeBuilder,
    },
    style::NoStyle,
    Result,
};

use super::{content::GlobalContent, event_comp::GlobalEventMgr, Window};

pub(super) async fn new_window<El, F>(window_builder: F) -> Result<Window>
where
    El: Element + ElementMutate<(), ()>,
    F: FnOnce(WindowBuilder) -> WindowBuilder + Send + 'static,
{
    let ev_disp = EventDispatcher::new();

    let create_app = {
        let ev_disp = ev_disp.clone();
        move |window: Arc<WinitWindow>, close_handle| BackendRuntime::<
            El,
            ChildrenCache<El, (), ()>,
        > {
            window,
            application: None,
            global_event_mgr: GlobalEventMgr::new(ev_disp),
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

pub(super) struct BackendRuntime<El, Cc> {
    window: Arc<WinitWindow>,
    application: Option<AddChildCache<El, Cc>>,
    global_event_mgr: GlobalEventMgr,
    close_handle: CloseHandle,
    layer_compositer: SharedLayerCompositer,
}

impl<El> AppWindow for BackendRuntime<El, ChildrenCache<El, (), ()>>
where
    El: Element + ElementMutate<(), ()>,
{
    fn on_redraw(&mut self, canvas: &mut Canvas, size: (u32, u32), delta: Duration) -> Result<()> {
        let add_child = add_child::<El, _, _, _, _>((), NoStyle, (), |_: &_| {});

        let region = (
            Point(Pixel(0.0), Pixel(0.0)),
            Point(
                Pixel::from_physical(size.0 as _),
                Pixel::from_physical(size.1 as _),
            ),
        );

        let mut lc = self.layer_compositer.borrow_mut();

        if self.application.is_none() {
            let mut rebuilder = lc.rebuild(canvas);

            let content = GlobalContent {
                global_ed: &self.global_event_mgr.global_ed(),
                focusing: self.global_event_mgr.focusing(),
                window: &self.window,
                close_handle: self.close_handle,
                interval: delta,
            };

            TreeBuilder::new(add_child, &mut self.application, content, false).finish(region)?;
        }

        canvas.clear(TRANSPARENT);
        lc.composite(canvas)
    }

    fn on_window_event(&mut self, event: StaticWindowEvent) {
        let new_event = self.global_event_mgr.emit_event(event);
    }
}
