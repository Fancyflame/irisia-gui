use std::{future::Future, sync::mpsc};

use lazy_static::lazy_static;
use tokio::{runtime::Runtime, task::spawn_blocking};
use winit::{
    event::{Event, StartCause},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
    window::{WindowBuilder, WindowId},
};

use self::{
    global::WindowRegiterMutex,
    rt_event::{RuntimeEvent, WindowReg},
    send_event::WindowMap,
};

pub(crate) mod global;
pub(crate) mod rt_event;
mod send_event;

lazy_static! {
    pub static ref TOKIO_RT: Runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
}

pub fn start_runtime<F>(f: F) -> !
where
    F: FnOnce() + Send + 'static,
{
    let mut func_option = Some(f);

    let event_loop: EventLoop<WindowReg> = EventLoopBuilder::with_user_event().build();
    let mut window_map = WindowMap::new();
    WindowRegiterMutex::init(event_loop.create_proxy());

    event_loop.run(move |event, event_loop, flow| {
        match event {
            Event::NewEvents(StartCause::Init) => {
                let func = func_option.take().expect("unexpected take function twice");
                /*std::thread::spawn(move || lazy_static::initialize(&TOKIO_RT))
                .join()
                .unwrap();*/
                std::thread::spawn(func);
            }

            Event::WindowEvent { window_id, .. } | Event::RedrawRequested(window_id) => {
                window_map.send_sys_event(window_id, event);
            }

            Event::UserEvent(WindowReg::WindowCreate { build, sender }) => {
                let win = build(WindowBuilder::new()).build(event_loop);
                if let Ok(win) = &win {
                    window_map.insert(win.id(), sender.clone());
                }
                let _ = sender.send(RuntimeEvent::WindowCreated { win });
            }

            Event::UserEvent(WindowReg::WindowDestroyed(wid)) => {
                window_map.remove(&wid);
            }

            _ => {}
        }
        flow.set_poll();
    });
}

fn start_tokio_rt<F>(block_fn: F)
where
    F: FnOnce() + Send + 'static,
{
    lazy_static::initialize(&TOKIO_RT);
    block_fn();
}
