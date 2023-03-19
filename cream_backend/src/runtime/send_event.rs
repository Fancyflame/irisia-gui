use std::collections::HashMap;

use std::ops::{Deref, DerefMut};

use super::*;

pub(crate) type WindowMapInner = HashMap<WindowId, mpsc::Sender<RuntimeEvent>>;

pub(super) struct WindowMap(WindowMapInner);

impl WindowMap {
    pub fn new() -> Self {
        WindowMap(HashMap::new())
    }

    pub(super) fn send_sys_event(&mut self, wid: WindowId, event: Event<WindowReg>) {
        if let Some(event) = event.to_static() {
            let event = RuntimeEvent::SysEvent(
                event
                    .map_nonuser_event()
                    .map_err(|_| "inner error: cannot send user event")
                    .unwrap(),
            );

            self.send_event(wid, event);
        }
    }

    pub(super) fn send_sys_event_all(&mut self, event: Event<WindowReg>) {
        if let Some(event) = event.to_static() {
            let event = event
                .map_nonuser_event()
                .map_err(|_| "inner error: cannot send user event")
                .unwrap();
            self.retain(|_, v| v.send(RuntimeEvent::SysEvent(event.clone())).is_ok())
        }
    }

    pub(super) fn send_event(&mut self, wid: WindowId, event: RuntimeEvent) {
        match self.get(&wid) {
            Some(sender) => {
                if let RuntimeEvent::SysEvent(Event::WindowEvent { event, .. }) = &event {
                    dbg!(event);
                    println!("send");
                }
                if sender.send(event).is_err() {
                    self.remove(&wid);
                }
            }
            None => {
                eprintln!("no window id matched, skipped.");
            }
        }
    }
}

impl Deref for WindowMap {
    type Target = WindowMapInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for WindowMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
