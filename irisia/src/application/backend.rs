use std::{cell::RefCell, rc::Rc, sync::Arc, time::Duration};

use irisia_backend::{
    skia_safe::{colors::WHITE, Canvas},
    window_handle::{RawWindowHandle, WindowBuilder},
    winit::{dpi::PhysicalSize, event::WindowEvent},
    AppWindow, WinitWindow,
};

use crate::{
    dom::ElementModel,
    element::Element,
    event::{standard::WindowDestroyed, EventDispatcher},
    primitive::{Pixel, Point, Region},
    Result,
};

use super::{
    content::GlobalContent,
    event_comp::{global::focusing::Focusing, GlobalEventMgr},
    redraw_scheduler::RedrawScheduler,
    Window,
};

pub(super) struct BackendRuntime<El: Element> {
    gem: GlobalEventMgr,
    gc: Rc<GlobalContent>,
    root_element: DropProtection<El, (), ()>,
}

impl<El> AppWindow for BackendRuntime<El>
where
    El: Element,
{
    fn on_redraw(&mut self, canvas: &Canvas, interval: Duration) -> Result<()> {
        self.gc
            .redraw_scheduler
            .borrow_mut()
            .redraw(canvas, interval)?;

        // composite
        canvas.reset_matrix();
        canvas.clear(WHITE);
        self.root_element.composite_as_root(canvas)
    }

    fn on_window_event(&mut self, event: WindowEvent) {
        if let WindowEvent::Resized(size) = &event {
            self.root_element
                .set_draw_region(window_size_to_draw_region(*size));
        }

        if let Some(ipe) = self.gem.emit_event(event, &self.gc) {
            if !self.root_element.emit_event(&ipe) {
                ipe.focus_on(None);
            }
        }
    }

    fn on_destroy(&mut self) {
        self.gc.event_dispatcher().emit_trusted(WindowDestroyed);
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
    El: Element + From<()>,
    F: FnOnce(WindowBuilder) -> WindowBuilder + Send + 'static,
{
    let ev_disp = EventDispatcher::new();

    let create_app = {
        let ev_disp = ev_disp.clone();

        move |window: Arc<WinitWindow>, close_handle| {
            let redraw_scheduler = RedrawScheduler::new(window.clone());

            let gc = Rc::new(GlobalContent {
                global_ed: ev_disp,
                focusing: Focusing::new(),
                window,
                redraw_scheduler: RefCell::new(redraw_scheduler),
                close_handle,
            });

            let root_element = ElementModel::new((), (), ());

            root_element.set_draw_region(window_size_to_draw_region(gc.window().inner_size()));
            gc.request_redraw(root_element.0.clone());

            BackendRuntime::<El> {
                root_element,
                gem: GlobalEventMgr::new(),
                gc,
            }
        }
    };

    let RawWindowHandle {
        raw_window,
        close_handle,
    } = RawWindowHandle::create(create_app, window_builder).await?;

    Ok(Window {
        winit_window: Arc::downgrade(&raw_window),
        close_handle,
        event_dispatcher: ev_disp,
    })
}
