use irisia_macros::__inner_impl_listen;
use std::future::Future;
use tokio::task::JoinHandle;

use crate::{
    event::{EventDispatcher, EventReceiver, SubEvent},
    Event,
};

#[derive(Debug, Clone)]
pub struct Listen<'a, Ep, T0 = (), T1 = (), T2 = (), T3 = (), T4 = ()> {
    ep: &'a Ep,
    once: T0,
    trusted: T1,
    asyn: T2,
    sub_event: T3,
    no_handle: T4,
}

#[derive(Default, Clone, Copy)]
pub struct FlagSet;

impl<'a, Ep> Listen<'a, Ep> {
    pub(crate) fn new(ep: &'a Ep) -> Self {
        Listen {
            ep,
            once: (),
            trusted: (),
            asyn: (),
            sub_event: (),
            no_handle: (),
        }
    }
}

macro_rules! auto_fn {
    ($($name:ident: $t0:ident $t1:ident $t2:ident $t3:ident $t4:ident,)*) => {
        $(
            pub fn $name(self) -> Listen<'a, Ep, $t0, $t1, $t2, $t3, $t4> {
                Listen {
                    ep: self.ep,
                    once: choose_value!($t0 self.once),
                    trusted: choose_value!($t1 self.trusted),
                    asyn: choose_value!($t2 self.asyn),
                    sub_event: choose_value!($t3 self.sub_event),
                    no_handle: choose_value!($t4 self.no_handle),
                }
            }
        )*
    };
}

macro_rules! choose_value {
    (FlagSet $expr:expr) => {
        FlagSet
    };
    ($_:ident $expr:expr) => {
        $expr
    };
}

impl<'a, Ep, T0, T1, T2, T3, T4> Listen<'a, Ep, T0, T1, T2, T3, T4> {
    auto_fn! {
        once:           FlagSet T1 T2 T3 T4,
        trusted:        T0 FlagSet T2 T3 T4,
        asyn:           T0 T1 FlagSet T3 T4,
        sub_event:      T0 T1 T2 FlagSet T4,
        no_handle:      T0 T1 T2 T3 FlagSet,
    }
}

pub trait EdProvider: Clone + 'static {
    fn event_dispatcher(&self) -> &EventDispatcher;
    fn daemon<F>(&self, f: F) -> JoinHandle<()>
    where
        F: Future + 'static;
    fn handle_available(&self) -> bool;
}

impl EdProvider for EventDispatcher {
    fn event_dispatcher(&self) -> &EventDispatcher {
        self
    }

    fn daemon<F>(&self, f: F) -> JoinHandle<()>
    where
        F: Future + 'static,
    {
        tokio::task::spawn_local(async move {
            f.await;
        })
    }

    fn handle_available(&self) -> bool {
        true
    }
}

__inner_impl_listen!();
