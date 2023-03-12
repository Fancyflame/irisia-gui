use std::sync::mpsc;

use winit::{
    event::Event,
    event_loop::{EventLoop, EventLoopBuilder},
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

pub fn start_runtime() -> ! {
    let event_loop: EventLoop<WindowReg> = EventLoopBuilder::with_user_event().build();
    let mut window_map = WindowMap::new();
    WindowRegiterMutex::init(event_loop.create_proxy());

    event_loop.run(move |event, event_loop, _flow| match event {
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

        _ => window_map.send_sys_event_all(event),
    });
}
