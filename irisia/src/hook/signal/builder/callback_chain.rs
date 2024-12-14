use std::rc::Weak;

use crate::hook::{listener::CallbackAction, provider_group::ProviderGroup, Listener};

use super::{Inner, Setter};

pub trait CallbackChain<T, C> {
    fn listen<F>(&self, src: Weak<Inner<T, C>>, get_node: F)
    where
        F: Fn(&Inner<T, C>) -> &Self + Copy + 'static;
}

impl<T, C> CallbackChain<T, C> for () {
    fn listen<F>(&self, _: Weak<Inner<T, C>>, _: F)
    where
        F: Fn(&Inner<T, C>) -> &Self + Copy + 'static,
    {
    }
}

pub struct CallbackNode<F, D, Next> {
    pub(super) deps: D,
    pub(super) callback: F,
    pub(super) next: Next,
}

impl<T, C, F, D, Next> CallbackChain<T, C> for CallbackNode<F, D, Next>
where
    T: 'static,
    C: 'static,
    D: ProviderGroup + 'static,
    F: Fn(Setter<T>, D::Data<'_>) + 'static,
    Next: CallbackChain<T, C>,
{
    fn listen<Fg>(&self, weak_src: Weak<Inner<T, C>>, get_node: Fg)
    where
        Fg: Fn(&Inner<T, C>) -> &Self + Copy + 'static,
    {
        let listener = Listener::new(weak_src.clone(), move |src, action| {
            if !action.is_update() {
                src.listeners.callback_all(action);
                return;
            }

            let this = get_node(src);
            let mut mutated = false;

            (this.callback)(
                Setter {
                    r: &mut src.value.borrow_mut().unwrap(),
                    mutated: &mut mutated,
                },
                D::deref_wrapper(&this.deps.read_many()),
            );

            src.listeners.callback_all(if mutated {
                CallbackAction::Update
            } else {
                CallbackAction::ClearDirty
            });
        });
        self.deps.dependent_many(listener);
        self.next.listen(weak_src, move |src| &get_node(src).next);
    }
}
