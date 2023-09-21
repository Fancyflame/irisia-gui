use std::{cell::RefCell, rc::Rc, sync::Arc, time::Duration};

use irisia_backend::{
    skia_safe::{colors::TRANSPARENT, Canvas},
    window_handle::{RawWindowHandle, WindowBuilder},
    winit::dpi::PhysicalSize,
    AppWindow, StaticWindowEvent, WinitWindow,
};

use crate::{
    dom::{add_one, update::ElementModelUpdater, EMUpdateContent},
    element::{Element, ElementUpdate, RcElementModel},
    event::EventDispatcher,
    primitive::{Pixel, Point, Region},
    update_with::UpdateWith,
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
    root_element: RcElementModel<El, (), ()>,
}

impl<El> AppWindow for BackendRuntime<El>
where
    El: Element,
{
    fn on_redraw(&mut self, canvas: &mut Canvas, interval: Duration) -> Result<()> {
        self.gc
            .redraw_scheduler
            .borrow_mut()
            .redraw(canvas, interval)?;

        // composite
        canvas.reset_matrix();
        canvas.clear(TRANSPARENT);
        self.root_element.composite(canvas)
    }

    fn on_window_event(&mut self, event: StaticWindowEvent) {
        if let StaticWindowEvent::Resized(size) = &event {
            self.root_element
                .set_draw_region(window_size_to_draw_region(*size));
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
    El: Element + ElementUpdate<()>,
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

            let root_element = <RcElementModel<El, (), ()> as UpdateWith<
                ElementModelUpdater<'_, El, (), (), (), _>,
            >>::create_with(ElementModelUpdater {
                add_one: add_one((), (), (), |_: &_| {}),
                content: EMUpdateContent {
                    global_content: &gc,
                    parent_layer: None,
                },
            });

            root_element.set_draw_region(window_size_to_draw_region(gc.window().inner_size()));

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
        winit_window: raw_window,
        close_handle,
        event_dispatcher: ev_disp,
    })
}
