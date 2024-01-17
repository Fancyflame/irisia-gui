use irisia_macros::__inner_impl_listen;
use std::{future::Future, marker::PhantomData};
use tokio::task::JoinHandle;

use crate::{
    event::{EventDispatcher, EventReceiver, SubEvent},
    Event,
};

#[derive(Debug)]
pub struct Listen<'a, Ep, Once, Trusted, Async, SubEv, WithHd> {
    ep: &'a Ep,
    _phantom: PhantomData<(Once, Trusted, Async, SubEv, WithHd)>,
}

impl<Ep, T0, T1, T2, T3, T4> Clone for Listen<'_, Ep, T0, T1, T2, T3, T4> {
    fn clone(&self) -> Self {
        Self {
            ep: self.ep,
            _phantom: PhantomData,
        }
    }
}

impl<Ep, T0, T1, T2, T3, T4> Copy for Listen<'_, Ep, T0, T1, T2, T3, T4> {}

pub struct FlagSet(());

pub trait ListenerOption<Ep, Event, Once, Trusted, Async, SubEv, WithHd>
where
    Ep: EdProvider,
{
    fn listen_from(self, l: Listen<Ep, Once, Trusted, Async, SubEv, WithHd>) -> JoinHandle<()>;
}

impl<'a, Ep, T0, T1, T2, T3, T4> Listen<'a, Ep, T0, T1, T2, T3, T4> {
    pub(crate) fn new(ep: &'a Ep) -> Self {
        Listen {
            ep,
            _phantom: PhantomData,
        }
    }

    pub fn once(self) -> Listen<'a, Ep, FlagSet, T1, T2, T3, T4> {
        Listen::new(self.ep)
    }

    pub fn trusted(self) -> Listen<'a, Ep, T0, FlagSet, T2, T3, T4> {
        Listen::new(self.ep)
    }
}

impl<'a, Ep, T0, T1, T2, T3, T4> Listen<'a, Ep, T0, T1, T2, T3, T4> {
    pub fn spawn<F, E>(&self, trigger: F) -> JoinHandle<()>
    where
        Ep: EdProvider,
        F: ListenerOption<Ep, E, T0, T1, T2, T3, T4>,
    {
        trigger.listen_from(*self)
    }
}

macro_rules! auto_fn {
    ($($name:ident: <$($impl_gen: tt),*> <$($st_gen: tt),*>,)*) => {
        $(
            impl<Ep, T0, T1, $($impl_gen),*> Listen<'_, Ep, T0, T1, $($st_gen),*> {
                pub fn $name(self) -> Self {
                    self
                }
            }
        )*
    };
}

auto_fn! {
    asyn:            <T3, T4> <FlagSet, T3, T4>,
    sub_event:       <T2, T4> <T2, FlagSet, T4>,
    with_handle:     <T2, T3> <T2, T3, FlagSet>,
    sync:            <T3, T4> <(), T3, T4>,
    normal_event:    <T2, T4> <T2, (), T4>,
    no_handle:       <T2, T3> <T2, T3, ()>,
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

#[cfg(test)]
#[allow(unused)]
fn test(ed: EventDispatcher) {
    type Ev = super::standard::PointerMove;
    ed.listen().once().spawn(|_: Ev| {});
    ed.listen().spawn(|_: Ev, _: EventDispatcher| async {});
    ed.listen().asyn().spawn(|_: Ev| async {});
}
