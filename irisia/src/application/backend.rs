use std::{cell::Cell, rc::Rc, sync::Arc, time::Duration};

use irisia_backend::{
    skia_safe::Canvas,
    window_handle::RawWindowHandle,
    winit::{dpi::PhysicalSize, event::WindowEvent, window::WindowAttributes},
    AppWindow, WinitWindow,
};

use crate::{
    event::{standard::WindowDestroyed, EventDispatcher},
    model::VNode,
    prim_element::{EMCreateCtx, GetElement, RenderTree},
    primitive::{Point, Region},
    Result,
};

use super::{
    content::GlobalContent,
    event2::pointer_event::{PointerState, PointerStateDelta},
    event_comp::global::focusing::Focusing,
    redraw_scheduler::RedrawScheduler,
    Window,
};

pub(super) struct BackendRuntime<T> {
    pointer_state: PointerState,
    gc: Rc<GlobalContent>,
    root_model: T,
}

impl<T> AppWindow for BackendRuntime<T>
where
    T: GetElement + 'static,
{
    fn on_redraw(
        &mut self,
        canvas: &Canvas,
        interval: Duration,
        window_inner_size: PhysicalSize<u32>,
    ) -> Result<()> {
        self.gc.redraw_scheduler.redraw(
            &mut self.root_model.get_element(),
            canvas,
            interval,
            window_size_to_draw_region(window_inner_size),
        );
        Ok(())
    }

    fn on_window_event(&mut self, event: WindowEvent, window_inner_size: PhysicalSize<u32>) {
        let Some(next) = self.pointer_state.next(&event) else {
            // TODO
            return;
        };

        let mut delta = PointerStateDelta {
            prev: self.pointer_state,
            next,
            cursor_may_over: true,
        };
        self.pointer_state = next;

        self.root_model
            .get_element()
            .emit_event(&mut delta, window_size_to_draw_region(window_inner_size));
        // TODO
        // if let WindowEvent::Resized(size) = &event {
        //     self.root
        //         .set_draw_region(Some(window_size_to_draw_region(*size)));
        // }

        // if let Some(ipe) = self.gem.emit_event(event, &self.gc) {
        //     if !self.root.on_pointer_event(&ipe) {
        //         ipe.focus_on(None);
        //     }
        // }
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
    T: VNode,
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

            let root_model = root_creator().create(
                0,
                &EMCreateCtx {
                    global_content: gc.clone(),
                },
            );

            //root.set_draw_region(Some(window_size_to_draw_region(gc.window().inner_size())));
            // root.

            BackendRuntime {
                pointer_state: PointerState::new(),
                gc,
                root_model,
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
