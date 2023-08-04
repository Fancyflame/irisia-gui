use std::{sync::Arc, time::Duration};

use irisia_backend::{
    skia_safe::{colors::TRANSPARENT, Canvas},
    window_handle::{RawWindowHandle, WindowBuilder},
    AppWindow, StaticWindowEvent, WinitWindow,
};

use crate::{
    dom::{
        add_one,
        layer::{LayerCompositer, SharedLayerCompositer},
        ElementModel,
    },
    element::Element,
    event::EventDispatcher,
    primitive::{Pixel, Point},
    Result, UpdateWith,
};

use super::{
    content::GlobalContent,
    event_comp::{global::focusing::Focusing, GlobalEventMgr},
    EmptyUpdateOptions, Window,
};

pub(super) async fn new_window<El, F>(window_builder: F) -> Result<Window>
where
    El: Element + for<'a> UpdateWith<EmptyUpdateOptions<'a>>,
    F: FnOnce(WindowBuilder) -> WindowBuilder + Send + 'static,
{
    let ev_disp = EventDispatcher::new();

    let create_app = {
        let ev_disp = ev_disp.clone();

        move |window: Arc<WinitWindow>, close_handle| {
            let gc = Arc::new(GlobalContent {
                global_ed: ev_disp,
                focusing: Focusing::new(),
                window,
                close_handle,
            });

            BackendRuntime::<El> {
                root_element: ElementModel::create_with((add_one((), (), (), |_: &_| {}), &gc)),
                gem: GlobalEventMgr::new(),
                gc,
                layer_compositer: LayerCompositer::new_shared(),
            }
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
    gem: GlobalEventMgr,
    gc: Arc<GlobalContent>,
    root_element: ElementModel<El, ()>,
    layer_compositer: SharedLayerCompositer,
}

impl<El> AppWindow for BackendRuntime<El>
where
    El: Element + for<'a> UpdateWith<EmptyUpdateOptions<'a>>,
{
    fn on_redraw(&mut self, canvas: &mut Canvas, size: (u32, u32), delta: Duration) -> Result<()> {
        let region = (
            Point(Pixel(0.0), Pixel(0.0)),
            Point(
                Pixel::from_physical(size.0 as _),
                Pixel::from_physical(size.1 as _),
            ),
        );

        let mut lc = self.layer_compositer.borrow_mut();
        self.root_element
            .render(&mut lc.rebuild(canvas), region, delta)?;

        // composite
        canvas.clear(TRANSPARENT);
        lc.composite(canvas)
    }

    fn on_window_event(&mut self, event: StaticWindowEvent) {
        if let Some(npe) = self.gem.emit_event(event, &self.gc) {
            if !self.root_element.emit_event(&npe) {
                npe.focus_on(None);
            }
        }
    }
}
