use std::{cell::Cell, rc::Rc, sync::Arc, time::Duration};

use irisia_backend::{
    skia_safe::Canvas,
    window_handle::RawWindowHandle,
    winit::{dpi::PhysicalSize, event::WindowEvent, window::WindowAttributes},
    AppWindow, WinitWindow,
};

use crate::{
    el_model::{EMCreateCtx, ElementModel},
    element::ElementInterfaces,
    event::{standard::WindowDestroyed, EventDispatcher},
    model::RootDesiredModel,
    primitive::{Point, Region},
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
    root: ElementModel<El, (), ()>,
}

impl<El> AppWindow for BackendRuntime<El>
where
    El: ElementInterfaces,
{
    fn on_redraw(&mut self, canvas: &Canvas, interval: Duration) -> Result<()> {
        self.gc
            .redraw_scheduler
            .redraw(&mut self.root, canvas, interval)
    }

    fn on_window_event(&mut self, event: WindowEvent) {
        if let WindowEvent::Resized(size) = &event {
            self.root
                .set_draw_region(Some(window_size_to_draw_region(*size)));
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
    Region {
        left_top: Point(0.0, 0.0),
        right_bottom: Point(size.width as f32, size.height as f32),
    }
}

pub(super) async fn new_window<F, T>(
    window_attributes: WindowAttributes,
    root_creator: F,
) -> Result<Window>
where
    F: FnOnce() -> T + Send + 'static,
    T: RootDesiredModel<(), RootCp = (), RootSlt = ()>,
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
                user_close: Cell::new(true),
            });

            let mut root = root_creator().create(&EMCreateCtx {
                global_content: gc.clone(),
            });

            root.set_draw_region(Some(window_size_to_draw_region(gc.window().inner_size())));

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
    } = RawWindowHandle::create(create_app, window_attributes).await?;

    Ok(Window {
        winit_window: Arc::downgrade(&raw_window),
        close_handle,
        event_dispatcher: ev_disp,
    })
}
