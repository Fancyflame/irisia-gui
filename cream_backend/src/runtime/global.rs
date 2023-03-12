use std::sync::{Mutex, MutexGuard};

use winit::event_loop::EventLoopProxy;

use super::rt_event::WindowReg;

static WINDOW_REGITER: Mutex<Option<EventLoopProxy<WindowReg>>> = Mutex::new(None);

pub(crate) struct WindowRegiterMutex(MutexGuard<'static, Option<EventLoopProxy<WindowReg>>>);

impl WindowRegiterMutex {
    pub fn init(proxy: EventLoopProxy<WindowReg>) {
        if Self::lock().0.replace(proxy).is_some() {
            panic!("global event loop has been initialized");
        }
    }

    pub fn lock() -> Self {
        WindowRegiterMutex(WINDOW_REGITER.lock().expect("cannot get mutex"))
    }

    pub fn send(&self, reg: WindowReg) {
        self.0
            .as_ref()
            .expect("event loop not started")
            .send_event(reg)
            .map_err(|e| format!("{e}"))
            .unwrap();
    }
}
