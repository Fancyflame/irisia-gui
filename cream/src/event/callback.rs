use crate::map_rc::MapWeak;

use super::event_flow::EventFlow;

mod private {
    use crate::event::EventFlow;

    pub enum Callback<S, T> {
        Full(fn(&S, &T, &mut EventFlow)),
        Short(fn(&S, &T)),
    }

    impl<S, T> From<fn(&S, &T, &mut EventFlow)> for Callback<S, T> {
        fn from(f: fn(&S, &T, &mut EventFlow)) -> Self {
            Callback::Full(f)
        }
    }

    impl<S, T> From<fn(&S, &T)> for Callback<S, T> {
        fn from(f: fn(&S, &T)) -> Self {
            Callback::Short(f)
        }
    }

    impl<S, T> Callback<S, T> {
        pub(crate) fn call(&self, slf: &S, args: &T, flow: &mut EventFlow) {
            match self {
                Callback::Full(f) => f(slf, args, flow),
                Callback::Short(f) => f(slf, args),
            }
        }
    }
}

pub struct Closure<S, T> {
    slf: MapWeak<S>,
    func: private::Callback<S, T>,
}

impl<S, T> Closure<S, T> {
    pub fn new<F>(slf: MapWeak<S>, func: F) -> Self
    where
        F: Into<private::Callback<S, T>>,
    {
        Closure {
            slf,
            func: func.into(),
        }
    }
}

pub(crate) trait ClosureCall<T> {
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

        self.func.call(&slf, args, flow);
    }
}
