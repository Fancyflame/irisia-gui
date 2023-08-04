use std::sync::{Arc, Mutex as StdMutex};

use crate::event::{
    standard::{Blured, Focused},
    EventDispatcher,
};

pub struct Focusing(Arc<StdMutex<Option<EventDispatcher>>>);

impl Focusing {
    pub fn new() -> Self {
        Focusing(Default::default())
    }

    pub fn focus(&self, ed: EventDispatcher) {
        let mut guard = self.0.lock().unwrap();

        if let Some(old_ed) = &*guard {
            if ed.as_ptr() == old_ed.as_ptr() {
                return;
            }
        }

        blur(&mut guard);
        ed.emit_sys(Focused);
        *guard = Some(ed);
    }

    pub fn blur(&self) {
        blur(&mut self.0.lock().unwrap())
    }

    pub fn blur_checked(&self, ed: &EventDispatcher) {
        let mut guard = self.0.lock().unwrap();
        if let Some(focused) = &mut *guard {
            if focused.is_same(ed) {
                blur(&mut guard);
            }
        }
    }
}

fn blur(ed: &mut Option<EventDispatcher>) {
    if let Some(ed) = ed.take() {
        ed.emit_sys(Blured);
    }
}
