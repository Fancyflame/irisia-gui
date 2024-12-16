use crate::{coerce_signal, hook::provider_group::ProviderGroup};

use super::Signal;

#[derive(Clone)]
pub struct Effect(Signal<dyn Noop>);

struct Inner<T, F, Fd: FnOnce()> {
    state: T,
    make_effect: F,
    drop_guard: DropGuard<Fd>,
}

trait Noop {}
impl<T> Noop for T {}

struct DropGuard<Fd: FnOnce()>(Option<Fd>);
impl<Fd: FnOnce()> Drop for DropGuard<Fd> {
    fn drop(&mut self) {
        if let Some(dropper) = self.0.take() {
            dropper();
        }
    }
}

impl Effect {
    pub fn effect<T, F, Fd, D>(mut init_state: T, make_effect: F, deps: D) -> Self
    where
        T: 'static,
        F: Fn(&mut T, D::Data<'_>) -> Fd + 'static,
        Fd: FnOnce() + 'static,
        D: ProviderGroup + 'static,
    {
        let drop_guard = DropGuard(Some(make_effect(
            &mut init_state,
            D::deref_wrapper(&deps.read_many()),
        )));

        let signal = Signal::builder(Inner {
            state: init_state,
            make_effect,
            drop_guard,
        })
        .dep(
            move |mut this, data| {
                let refm = &mut *this;
                refm.drop_guard.0.take().unwrap()(); // run dropper
                let new_dropper = (refm.make_effect)(&mut refm.state, data);
                refm.drop_guard.0 = Some(new_dropper);
            },
            deps,
        )
        .build();

        Effect(coerce_signal!(signal))
    }
}
