use std::{cell::RefCell, rc::Weak};

use crate::hook::{reactive::CallbackFnAlias, signal_group::SignalGroup, Listener};

use super::{Inner, ReactiveRef, RealRef};

pub trait CallbackChain<T>: 'static {
    fn listen<F>(&self, src: Weak<Inner<T>>, get_node: F)
    where
        F: Fn(&Inner<T>) -> &Self + Copy + 'static;
}

impl<T> CallbackChain<T> for () {
    fn listen<F>(&self, _: Weak<Inner<T>>, _: F)
    where
        F: Fn(&Inner<T>) -> &Self + Copy + 'static,
    {
    }
}

pub struct CallbackNode<F, D, Next> {
    pub(super) deps: D,
    pub(super) callback: RefCell<F>,
    pub(super) next: Next,
}

impl<T, F, D, Next> CallbackChain<T> for CallbackNode<F, D, Next>
where
    T: 'static,
    D: SignalGroup + 'static,
    F: CallbackFnAlias<T, D>,
    Next: CallbackChain<T>,
{
    fn listen<Fg>(&self, weak_src: Weak<Inner<T>>, get_node: Fg)
    where
        Fg: Fn(&Inner<T>) -> &Self + Copy + 'static,
    {
        let listener = Listener::new({
            let weak_src = weak_src.clone();
            move |action| {
                let Some(inner) = weak_src.upgrade() else {
                    return false;
                };

                if !action.is_update() {
                    return true;
                }

                let this = get_node(&inner);

                // if value is already borrowed, then add the callback to the queue
                let reactive_ref = match inner.value.try_borrow_mut() {
                    Some(value) => ReactiveRef::Real(RealRef::new(&inner.value, value)),
                    None => {
                        inner.delay_callbacks.borrow_mut().push_back(Box::new(
                            move |inner, value| {
                                let this = get_node(&inner);
                                this.callback.borrow_mut()(
                                    value,
                                    D::deref_wrapper(&this.deps.read_many()),
                                );
                            },
                        ));
                        return true;
                    }
                };

                let mut callback = this.callback.borrow_mut();
                callback(reactive_ref, D::deref_wrapper(&this.deps.read_many()));
                inner.recall_delayed_callback();

                true
            }
        });
        self.deps.dependent_many(listener);
        self.next.listen(weak_src, move |src| &get_node(src).next);
    }
}
