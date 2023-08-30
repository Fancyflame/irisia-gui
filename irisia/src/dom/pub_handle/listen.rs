use irisia_macros::__inner_impl_listen;
use std::{future::Future, ops::Deref};
use tokio::task::JoinHandle;

use crate::{
    element::ElementHandle,
    event::{EventReceiver, SubEvent},
    Event,
};

pub struct Listen<Eh, T0, T1, T2, T3, T4> {
    eh: Eh,
    once: T0,
    sys_only: T1,
    asyn: T2,
    sub_event: T3,
    without_handle: T4,
}

#[derive(Default)]
pub struct FlagSet;

impl<'a, Eh> Listen<Eh, (), (), (), (), ()> {
    pub(super) fn new(eh: Eh) -> Self {
        Listen {
            eh,
            once: (),
            sys_only: (),
            asyn: (),
            sub_event: (),
            without_handle: (),
        }
    }
}

macro_rules! auto_fn {
    ($($name:ident: $t0:ident $t1:ident $t2:ident $t3:ident $t4:ident,)*) => {
        $(
            pub fn $name(self) -> Listen<Eh, $t0, $t1, $t2, $t3, $t4> {
                Listen {
                    eh: self.eh,
                    once: choose_value!($t0 self.once),
                    sys_only: choose_value!($t1 self.sys_only),
                    asyn: choose_value!($t2 self.asyn),
                    sub_event: choose_value!($t3 self.sub_event),
                    without_handle: choose_value!($t4 self.without_handle),
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

impl<Eh, T0, T1, T2, T3, T4> Listen<Eh, T0, T1, T2, T3, T4> {
    auto_fn! {
        once:           FlagSet T1 T2 T3 T4,
        sys_only:       T0 FlagSet T2 T3 T4,
        asyn:           T0 T1 FlagSet T3 T4,
        sub_event:      T0 T1 T2 FlagSet T4,
        without_handle: T0 T1 T2 T3 FlagSet,
    }
}

__inner_impl_listen!();
