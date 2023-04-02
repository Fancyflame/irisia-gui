use lazy_static::lazy_static;
use tokio::sync::{Mutex, MutexGuard};

use winit::event_loop::EventLoopProxy;

use super::rt_event::WindowReg;

lazy_static! {
    static ref WINDOW_REGITER: Mutex<Option<EventLoopProxy<WindowReg>>> = Mutex::new(None);
}

pub(crate) struct WindowRegiterMutex(MutexGuard<'static, Option<EventLoopProxy<WindowReg>>>);

impl WindowRegiterMutex {
    pub fn init(proxy: EventLoopProxy<WindowReg>) {
        let mut guard = WINDOW_REGITER
            .try_lock()
            .expect("lock is unexpected blocked during initializing operation");

        if guard.replace(proxy).is_some() {
            panic!("global event loop has been initialized");
        }
    }

    pub async fn lock() -> Self {
        WindowRegiterMutex(WINDOW_REGITER.lock().await)
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
