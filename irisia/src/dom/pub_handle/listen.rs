use irisia_macros::__inner_impl_listen;
use std::{future::Future, rc::Rc};
use tokio::task::JoinHandle;

use crate::{
    dom::{ElementModel, RenderMultiple},
    event::{EventReceiver, SubEvent},
    style::StyleContainer,
    Element, Event,
};

pub struct Listen<Eh, T0, T1, T2, T3> {
    eh: Eh,
    once: T0,
    sys_only: T1,
    asyn: T2,
    sub_event: T3,
}

#[derive(Default)]
pub struct FlagSet;

impl<'a, Eh> Listen<Eh, (), (), (), ()> {
    pub(super) fn new(eh: Eh) -> Self {
        Listen {
            eh,
            once: (),
            sys_only: (),
            asyn: (),
            sub_event: (),
        }
    }
}

macro_rules! auto_fn {
    ($($name:ident: $t0:ident $t1:ident $t2:ident $t3:ident,)*) => {
        $(
            pub fn $name(self) -> Listen<Eh, $t0, $t1, $t2, $t3> {
                Listen {
                    eh: self.eh,
                    once: choose_value!($t0 self.once),
                    sys_only: choose_value!($t1 self.sys_only),
                    asyn: choose_value!($t2 self.asyn),
                    sub_event: choose_value!($t3 self.sub_event),
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

impl<Eh, T0, T1, T2, T3> Listen<Eh, T0, T1, T2, T3> {
    auto_fn! {
        once:           FlagSet T1 T2 T3,
        sys_only:       T0 FlagSet T2 T3,
        asyn:           T0 T1 FlagSet T3,
        sub_event:      T0 T1 T2 FlagSet,
    }
}

__inner_impl_listen!();
