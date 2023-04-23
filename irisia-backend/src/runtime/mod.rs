use std::{collections::HashMap, future::Future};

use winit::{
    event::{Event, StartCause},
    event_loop::{EventLoop, EventLoopBuilder},
    window::{WindowBuilder, WindowId},
};

use crate::render_window::RenderWindow;

use self::{global::WindowRegiterMutex, rt_event::WindowReg};

pub(crate) mod global;
pub(crate) mod rt_event;

pub async fn exit_app(code: i32) {
    WindowRegiterMutex::lock().await.send(WindowReg::Exit(code));
}

pub fn start_runtime<F>(f: F) -> !
where
    F: Future<Output = ()> + Send + 'static,
{
    let mut future_option = Some(async move {
        f.await;
        exit_app(0).await;
    });

    let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("cannot launch tokio runtime");
    let _guard = tokio_runtime.enter();

    let event_loop: EventLoop<WindowReg> = EventLoopBuilder::with_user_event().build();
    let mut window_map: HashMap<WindowId, RenderWindow> = HashMap::new();
    WindowRegiterMutex::init(event_loop.create_proxy());

    event_loop.run(move |event, event_loop, flow| {
        flow.set_poll();
        match event {
            Event::NewEvents(StartCause::Init) => {
                let future = future_option
                    .take()
                    .expect("unexpected take function twice");
                tokio_runtime.spawn(future);
            }

            Event::WindowEvent { window_id, .. } | Event::RedrawRequested(window_id) => {
                if let Some(w) = window_map.get_mut(&window_id) {
                    match event.map_nonuser_event() {
                        Ok(event) => w.handle_event(event),
                        _ => unreachable!(),
                    }
                }
            }

            Event::UserEvent(window_reg) => match window_reg {
                WindowReg::RawWindowRequest {
                    builder,
                    window_giver,
                } => {
                    let window = builder(WindowBuilder::new()).build(event_loop);
                    let _ = window_giver.send(window);
                }

                WindowReg::WindowRegister { app, raw_window } => {
                    let window_id = raw_window.id();

                    let render_window =
                        RenderWindow::new(app(), raw_window).expect("cannot load renderer");

                    window_map
                        .entry(window_id)
                        .or_insert(render_window)
                        .redraw();
                }

                WindowReg::WindowDestroyed(wid) => {
                    window_map.remove(&wid);
                }

                WindowReg::Exit(code) => {
                    flow.set_exit_with_code(code);
                }
            },

            _ => match event.map_nonuser_event() {
                Ok(e) => {
                    if let Some(e) = e.to_static() {
                        for window in window_map.values_mut() {
                            window.handle_event(e.clone());
                        }
                    }
                }
                _ => unreachable!(),
            },
        }
    });
}
