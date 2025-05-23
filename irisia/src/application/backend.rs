use std::{cell::Cell, rc::Rc, sync::Arc, time::Duration};

use irisia_backend::{
    AppWindow, WinitWindow,
    skia_safe::Canvas,
    window_handle::RawWindowHandle,
    winit::{dpi::PhysicalSize, event::WindowEvent, window::WindowAttributes},
};

use crate::{
    Result,
    event::{EventDispatcher, standard::WindowDestroyed},
    model::{EleModel, ModelCreateCtx, VNode},
    prim_element::{EMCreateCtx, EmitEventArgs, RenderTreeExt, callback_queue::CallbackQueue},
    primitive::{
        Point, Region,
        length::{LengthStandard, LengthStandardGlobalPart},
    },
};

use super::{
    Window,
    content::GlobalContent,
    event_comp::global::focusing::Focusing,
    event2::pointer_event::{PointerState, PointerStateDelta},
    redraw_scheduler::RedrawScheduler,
    window_size_to_constraint,
};

pub(super) struct BackendRuntime {
    pointer_state: PointerState,
    gc: Rc<GlobalContent>,
    root_model: Box<dyn EleModel>,
    callback_queue: CallbackQueue,
}

impl AppWindow for BackendRuntime {
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
            window_inner_size,
        );
        Ok(())
    }

    fn on_window_event(&mut self, event: WindowEvent, _window_inner_size: PhysicalSize<u32>) {
        // TODO: watch dpi change
        // if let WindowEvent::ScaleFactorChanged { scale_factor, inner_size_writer }

        if let WindowEvent::Resized(new_size) = event {
            let mut lsgp = self.gc.length_standard();
            lsgp.viewport_size = new_size.into();
            self.gc.length_standard.set(lsgp);

            self.root_model.get_element().borrow_mut().compute_layout(
                window_size_to_constraint(new_size),
                lsgp.viewport_size.map(|x| LengthStandard {
                    global: lsgp,
                    percentage_reference: x as f32,
                }),
            );
        }

        let Some(next) = self.pointer_state.next(&event) else {
            // TODO
            return;
        };

        let delta = PointerStateDelta {
            prev: self.pointer_state,
            next,
            cursor_may_over: true,
        };
        self.pointer_state = next;

        self.root_model
            .get_element()
            .borrow_mut()
            .emit_event(&mut EmitEventArgs {
                queue: &mut self.callback_queue,
                delta,
            });
        self.callback_queue.execute();
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
        left_top: Point { x: 0.0, y: 0.0 },
        right_bottom: Point {
            x: size.width as f32,
            y: size.height as f32,
        },
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
                length_standard: Cell::new(LengthStandardGlobalPart {
                    viewport_size: window.inner_size().into(),
                    dpi: window.scale_factor() as _,
                }),
                window,
                redraw_scheduler,
                close_handle,
                user_close: Cell::new(true),
            });

            let root_model = root_creator().create(&ModelCreateCtx::create_as_root(EMCreateCtx {
                global_content: gc.clone(),
                parent: None,
            }));

            //root.set_draw_region(Some(window_size_to_draw_region(gc.window().inner_size())));
            // root.

            BackendRuntime {
                pointer_state: PointerState::new(),
                gc,
                root_model: Box::new(root_model),
                callback_queue: CallbackQueue::new(),
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
