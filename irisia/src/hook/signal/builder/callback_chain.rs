use std::rc::Weak;

use crate::hook::{
    signal::inner::StrongListenerList, signal_group::SignalGroup, utils::CallbackAction, Listener,
};

use super::{Inner, Setter};

pub trait CallbackChain<T> {
    fn listen(self, src: Weak<Inner<T>>, store_list: &mut StrongListenerList);
}

impl<T> CallbackChain<T> for () {
    fn listen(self, _: Weak<Inner<T>>, _: &mut StrongListenerList) {}
}

pub struct CallbackNode<F, D, Next> {
    pub(super) deps: D,
    pub(super) callback: F,
    pub(super) next: Next,
}

impl<T, F, D, Next> CallbackChain<T> for CallbackNode<F, D, Next>
where
    T: 'static,
    D: SignalGroup + 'static,
    F: Fn(Setter<T>, D::Data<'_>) + 'static,
    Next: CallbackChain<T>,
{
    fn listen(self, weak_src: Weak<Inner<T>>, store_list: &mut StrongListenerList) {
        let strong_listener = Listener::new(|listener| {
            self.deps.dependent_many(listener);
            let weak_src = weak_src.clone();
            move |action| {
                let src = weak_src.upgrade().unwrap();

                if !action.is_update() {
                    src.push_action(action);
                    return true;
                }

                let mut mutated = false;

                (self.callback)(
                    Setter::new(
                        &mut src.value.borrow_mut().expect("failed updating signal"),
                        &mut mutated,
                    ),
                    D::deref_wrapper(&self.deps.read_many()),
                );

                src.push_action(if mutated {
                    CallbackAction::Update
                } else {
                    CallbackAction::ClearDirty
                });

                true
            }
        });

        store_list.push(strong_listener);
        self.next.listen(weak_src, store_list);
    }
}
