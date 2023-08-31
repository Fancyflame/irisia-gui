use std::{
    sync::{Arc, Mutex as StdMutex},
    time::Duration,
};

use irisia_backend::{
    skia_safe::{colors::TRANSPARENT, Canvas},
    window_handle::{RawWindowHandle, WindowBuilder},
    winit::dpi::PhysicalSize,
    AppWindow, StaticWindowEvent, WinitWindow,
};

use crate::{
    dom::{add_one, update::ElementModelUpdater, EMUpdateContent, ElementModel},
    element::Element,
    event::EventDispatcher,
    primitive::{Pixel, Point, Region},
    update_with::UpdateWith,
    Result,
};

use super::{
    content::GlobalContent,
    event_comp::{global::focusing::Focusing, GlobalEventMgr},
    redraw_scheduler::{RedrawScheduler, ROOT_LAYER_ID},
    EmptyUpdateOptions, Window,
};

pub(super) struct BackendRuntime<El: Element> {
    gem: GlobalEventMgr,
    gc: Arc<GlobalContent>,
    root_element: ElementModel<El, (), ()>,
    redraw_scheduler: RedrawScheduler,
}

impl<El> AppWindow for BackendRuntime<El>
where
    El: Element + for<'a> UpdateWith<EmptyUpdateOptions<'a, El>>,
{
    fn on_redraw(&mut self, canvas: &mut Canvas, interval: Duration) -> Result<()> {
        canvas.clear(TRANSPARENT);
        self.redraw_scheduler.redraw(
            canvas,
            |lr, reg, interval| self.root_element.render(lr, reg, interval),
            interval,
            &mut self.gc.redraw_list.lock().unwrap(),
        )?;

        // composite
        canvas.clear(TRANSPARENT);
        self.redraw_scheduler.composite(canvas)
    }

    fn on_window_event(&mut self, event: StaticWindowEvent) {
        if let StaticWindowEvent::Resized(size) = &event {
            self.root_element.layout(window_size_to_draw_region(*size));
        }

        if let Some(npe) = self.gem.emit_event(event, &self.gc) {
            if !self.root_element.emit_event(&npe) {
                npe.focus_on(None);
            }
        }
    }
}

fn window_size_to_draw_region(size: PhysicalSize<u32>) -> Region {
    (
        Point(Pixel(0.0), Pixel(0.0)),
        Point(
            Pixel::from_physical(size.width as _),
            Pixel::from_physical(size.height as _),
        ),
    )
}

pub(super) async fn new_window<El, F>(window_builder: F) -> Result<Window>
where
    El: Element + for<'a> UpdateWith<EmptyUpdateOptions<'a, El>>,
    F: FnOnce(WindowBuilder) -> WindowBuilder + Send + 'static,
{
    let ev_disp = EventDispatcher::new();

    let create_app = {
        let ev_disp = ev_disp.clone();

        move |window: Arc<WinitWindow>, close_handle| {
            let (redraw_scheduler, redraw_list) = RedrawScheduler::new(window.clone());

            let gc = Arc::new(GlobalContent {
                global_ed: ev_disp,
                focusing: Focusing::new(),
                window,
                redraw_list: StdMutex::new(redraw_list),
                close_handle,
            });

            let mut root_element = ElementModel::create_with(ElementModelUpdater {
                add_one: add_one((), (), (), |_: &_| {}),
                content: EMUpdateContent {
                    global_content: &gc,
                    dep_layer_id: ROOT_LAYER_ID,
                },
            });

            let window_size = gc.window().inner_size();

            root_element.layout(window_size_to_draw_region(window_size));

            BackendRuntime::<El> {
                root_element,
                gem: GlobalEventMgr::new(),
                gc,
                redraw_scheduler,
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
