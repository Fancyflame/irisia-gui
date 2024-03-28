use std::{cell::RefCell, rc::Rc, sync::Arc, time::Duration};

use irisia_backend::{
    skia_safe::{colors::WHITE, Canvas},
    window_handle::{RawWindowHandle, WindowBuilder},
    winit::{dpi::PhysicalSize, event::WindowEvent},
    AppWindow, WinitWindow,
};

use crate::{
    dom::{EMCreateCtx, ElementModel},
    element::{Element, ElementUpdate, NewState},
    event::{standard::WindowDestroyed, EventDispatcher},
    primitive::{Pixel, Point, Region},
    Result,
};

use super::{
    content::GlobalContent,
    event_comp::{global::focusing::Focusing, GlobalEventMgr},
    redraw_scheduler::RedrawScheduler,
    root::Root,
    Window,
};

pub(super) struct BackendRuntime<El: Element> {
    gem: GlobalEventMgr,
    gc: Rc<GlobalContent>,
    root: Rc<Root<El>>,
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
        self.root.composite(canvas)
    }

    fn on_window_event(&mut self, event: WindowEvent) {
        if let WindowEvent::Resized(size) = &event {
            self.root
                .el()
                .set_draw_region(window_size_to_draw_region(*size));
            self.gc.request_redraw(self.root.clone());
        }

        if let Some(ipe) = self.gem.emit_event(event, &self.gc) {
            if !self.root.el().on_pointer_event(&ipe) {
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

pub(super) async fn new_window<El>(window_builder: WindowBuilder) -> Result<Window>
where
    El: Element + ElementUpdate<El::BlankProps, (), ()> + From<()>,
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

            let root = Root::new(|weak| {
                El::create(
                    ElementModel::new(&EMCreateCtx {
                        global_content: gc.clone(),
                        parent_layer: weak,
                    }),
                    NewState {
                        props: <El::BlankProps as Default>::default(),
                        styles: (),
                        slot: (),
                    },
                )
            });

            root.el()
                .set_draw_region(window_size_to_draw_region(gc.window().inner_size()));

            BackendRuntime::<El> {
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
