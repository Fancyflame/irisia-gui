use crate::map_rc::MapWeak;

use super::event_flow::EventFlow;

pub struct Closure<S, T> {
    slf: MapWeak<S>,
    func: fn(&S, &T, &mut EventFlow),
}

impl<S, T> Closure<S, T> {
    pub fn new(slf: MapWeak<S>, func: fn(&S, &T, &mut EventFlow)) -> Self {
        Closure { slf, func }
    }
}

pub trait ClosureCall<T> {
    fn call(&self, args: &T, flow: &mut EventFlow);
}

impl<S, T> ClosureCall<T> for Closure<S, T> {
    fn call(&self, args: &T, flow: &mut EventFlow) {
        let slf = match self.slf.upgrade() {
            Some(s) => s,
            None => unreachable!(
                "`self.slf` is expected to be always valid during the closure's lifetime."
            ),
        };

        (self.func)(&slf, args, flow);
    }
}
