use std::{collections::HashMap, future::Future};

use tokio::task::LocalSet;
use winit::{
    event::{Event, StartCause},
    event_loop::{EventLoop, EventLoopBuilder},
    window::{WindowBuilder, WindowId},
};

use crate::render_window::RenderWindowController;

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
    let local_set = LocalSet::new();

    let _guards = (tokio_runtime.enter(), local_set.enter());

    let event_loop: EventLoop<WindowReg> = EventLoopBuilder::with_user_event().build();
    let mut window_map: HashMap<WindowId, RenderWindowController> = HashMap::new();
    WindowRegiterMutex::init(event_loop.create_proxy());

    event_loop.run(move |event, event_loop, flow| {
        flow.set_wait();

        match event {
            Event::NewEvents(StartCause::Init) => {
                let future = future_option
                    .take()
                    .expect("unexpected take function twice");
                tokio_runtime.spawn(future);
            }

            Event::WindowEvent { window_id, event } => {
                if let Some(w) = window_map.get_mut(&window_id) {
                    if let Err(err) = w.handle_event(event.to_static().unwrap()) {
                        println!("{err}");
                        window_map.remove(&window_id);
                    }
                }
            }

            Event::RedrawRequested(window_id) => {
                if let Some(w) = window_map.get_mut(&window_id) {
                    if let Err(err) = w.redraw() {
                        println!("{err}");
                        window_map.remove(&window_id);
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

                    let render_window = RenderWindowController::new(app, raw_window);
                    if let Err(err) = render_window.redraw() {
                        println!("{err}");
                        return;
                    }

                    window_map.insert(window_id, render_window);
                }

                WindowReg::WindowDestroyed(wid) => {
                    window_map.remove(&wid);
                }

                WindowReg::Exit(code) => {
                    flow.set_exit_with_code(code);
                }
            },

            _ => {} /*_ => match event.map_nonuser_event() {
                        Ok(e) => {
                            if let Some(e) = e.to_static() {
                                window_map.retain(|_, window| {
                                    if let Err(err) = window.handle_event(e.clone()) {
                                        println!("{err}");
                                        false
                                    } else {
                                        true
                                    }
                                });
                            }
                        }
                        _ => unreachable!(),
                    },*/
        }
    });
}
