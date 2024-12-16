use std::rc::Weak;

use crate::hook::{listener::CallbackAction, provider_group::ProviderGroup, Listener};

use super::{Inner, Setter};

pub trait CallbackChain<T> {
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
    pub(super) callback: F,
    pub(super) next: Next,
}

impl<T, F, D, Next> CallbackChain<T> for CallbackNode<F, D, Next>
where
    T: 'static,
    D: ProviderGroup + 'static,
    F: Fn(Setter<T>, D::Data<'_>) + 'static,
    Next: CallbackChain<T>,
{
    fn listen<Fg>(&self, weak_src: Weak<Inner<T>>, get_node: Fg)
    where
        Fg: Fn(&Inner<T>) -> &Self + Copy + 'static,
    {
        let listener = Listener::new(weak_src.clone(), move |src, action| {
            if !action.is_update() {
                src.push_action(action);
                return;
            }

            let this = get_node(src);
            let mut mutated = false;

            (this.callback)(
                Setter::new(&mut src.value.borrow_mut().unwrap(), &mut mutated),
                D::deref_wrapper(&this.deps.read_many()),
            );

            src.push_action(if mutated {
                CallbackAction::Update
            } else {
                CallbackAction::ClearDirty
            });
        });
        self.deps.dependent_many(listener);
        self.next.listen(weak_src, move |src| &get_node(src).next);
    }
}
