use crate::{coerce_signal, hook::provider_group::ProviderGroup};

use super::Signal;

#[must_use = "if `Effect` drops immediately then the background runtime stops as well. \
    please consider using `let _effect = ...` to keep it alive or drop it manually."]
#[derive(Clone)]
pub struct Effect {
    _keep_alive: Signal<dyn Noop>,
}

struct Inner<F, Fd: FnOnce()> {
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
    pub fn new<F, Fd, D>(mut make_effect: F, deps: D) -> Self
    where
        F: FnMut(D::Data<'_>) -> Fd + 'static,
        Fd: FnOnce() + 'static,
        D: ProviderGroup + 'static,
    {
        let drop_guard = DropGuard(Some(make_effect(D::deref_wrapper(&deps.read_many()))));

        let effect = Signal::builder(Inner {
            make_effect,
            drop_guard,
        })
        .dep(
            move |mut this, data| {
                let Inner {
                    make_effect,
                    drop_guard,
                } = &mut *this;
                drop_guard.0.take().unwrap()(); // run dropper
                let new_dropper = make_effect(data);
                drop_guard.0 = Some(new_dropper);
            },
            deps,
        )
        .build();

        Effect {
            _keep_alive: coerce_signal!(&effect),
        }
    }

    pub fn new_empty() -> Self {
        Effect {
            _keep_alive: coerce_signal!(&Signal::state(())),
        }
    }
}
