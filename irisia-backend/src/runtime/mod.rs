use std::{collections::HashMap, future::Future};

use anyhow::Result;
use tokio::task::LocalSet;
use winit::{
    application::ApplicationHandler,
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowId,
};

use crate::render_window::RenderWindowController;

use self::{global_event::WindowRegiterMutex, rt_event::WindowReg};

pub(crate) mod global_event;
pub(crate) mod rt_event;

pub async fn exit_app() {
    WindowRegiterMutex::lock().await.send(WindowReg::Exit);
}

pub fn start_runtime<F>(f: F) -> Result<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    #[cfg(feature = "dhat_heap")]
    let mut _profiler = Some(dhat::Profiler::new_heap());

    let mut future_option = Some(async move {
        f.await;
        exit_app().await;
    });

    let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("cannot launch tokio runtime");
    let local_set = LocalSet::new();

    let _guards = (tokio_runtime.enter(), local_set.enter());

    let event_loop: EventLoop<WindowReg> = EventLoop::with_user_event().build()?;
    let mut window_map: HashMap<WindowId, RenderWindowController> = HashMap::new();
    WindowRegiterMutex::init(event_loop.create_proxy());

    event_loop.run(move |event, window_target| {
        window_target.set_control_flow(ControlFlow::Wait);

        match event {
            Event::NewEvents(StartCause::Init) => {
                let future = future_option
                    .take()
                    .expect("unexpected take function twice");
                tokio_runtime.spawn(future);
            }

            Event::WindowEvent { window_id, event } => {
                if let Some(w) = window_map.get_mut(&window_id) {
                    let result = match event {
                        WindowEvent::RedrawRequested => w.redraw(),
                        _ => w.handle_event(event),
                    };

                    if let Err(err) = result {
                        println!("{err}");
                        window_map.remove(&window_id);
                    }
                }
            }

            Event::UserEvent(window_reg) => match window_reg {
                WindowReg::RawWindowRequest {
                    window_attributes,
                    window_giver,
                } => {
                    let result_window = window_target.create_window(window_attributes);
                    let _ = window_giver.send(result_window);
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

                WindowReg::Exit => {
                    #[cfg(feature = "dhat_heap")]
                    drop(_profiler.take());

                    window_target.exit();
                }
            },

            _ => {}
        }
    })?;

    Ok(())
}
