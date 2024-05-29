use std::{rc::Rc, sync::Arc, time::Duration};

use irisia_backend::{
    skia_safe::{colors::WHITE, Canvas},
    window_handle::{RawWindowHandle, WindowBuilder},
    winit::{dpi::PhysicalSize, event::WindowEvent},
    AppWindow, WinitWindow,
};

use crate::{
    el_model::{EMCreateCtx, SharedEM},
    element::ElementInterfaces,
    event::{standard::WindowDestroyed, EventDispatcher},
    primitive::{Pixel, Point, Region},
    structure::StructureCreate,
    Result,
};

use super::{
    content::GlobalContent,
    event_comp::{global::focusing::Focusing, GlobalEventMgr},
    redraw_scheduler::RedrawScheduler,
    Window,
};

pub(super) struct BackendRuntime<El> {
    gem: GlobalEventMgr,
    gc: Rc<GlobalContent>,
    root: SharedEM<El>,
}

impl<El> AppWindow for BackendRuntime<El>
where
    El: ElementInterfaces,
{
    fn on_redraw(&mut self, canvas: &Canvas, interval: Duration) -> Result<()> {
        self.gc.redraw_scheduler.redraw(canvas, interval)?;

        // composite
        canvas.reset_matrix();
        canvas.clear(WHITE);

        self.root
            .shared
            .render_on
            .expect_independent()
            .borrow_mut()
            .composite(canvas)
    }

    fn on_window_event(&mut self, event: WindowEvent) {
        if let WindowEvent::Resized(size) = &event {
            self.root.set_draw_region(window_size_to_draw_region(*size));
            self.gc.request_redraw(self.root.clone());
        }

        if let Some(ipe) = self.gem.emit_event(event, &self.gc) {
            if !self.root.on_pointer_event(&ipe) {
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

pub(super) async fn new_window<El, F>(
    window_builder: WindowBuilder,
    root_creator: F,
) -> Result<Window>
where
    F: StructureCreate<Target = SharedEM<El>> + Send + 'static,
    El: ElementInterfaces,
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
                redraw_scheduler,
                close_handle,
            });

            let root = root_creator.create(&EMCreateCtx {
                global_content: gc.clone(),
                parent_layer: None,
            });

            root.set_draw_region(window_size_to_draw_region(gc.window().inner_size()));

            BackendRuntime {
                gem: GlobalEventMgr::new(),
                gc,
                root,
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
