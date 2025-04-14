use crate::{application::PointerEvent, prim_element::EventCallback};

pub struct CallbackQueue(Vec<(EventCallback, PointerEvent)>);

impl CallbackQueue {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, callback: &EventCallback, event: PointerEvent) {
        self.0.push((callback.clone(), event));
    }

    pub fn execute(&mut self) {
        for (callback, event) in self.0.drain(..) {
            callback.read()(event);
        }
    }
}

impl Extend<(EventCallback, PointerEvent)> for CallbackQueue {
    fn extend<T: IntoIterator<Item = (EventCallback, PointerEvent)>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}
