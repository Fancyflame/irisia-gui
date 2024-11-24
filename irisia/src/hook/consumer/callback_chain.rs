use std::rc::Weak;

use crate::hook::{provider_group::ProviderGroup, Listener};

use super::Inner;

pub trait CallbackChain<T> {
    fn listen<F>(&self, src: Weak<T>, get_node: F)
    where
        F: Fn(&T) -> &Self + Copy + 'static;
}

impl<T> CallbackChain<T> for () {
    fn listen<F>(&self, _: Weak<T>, _: F)
    where
        F: Fn(&T) -> &Self + Copy + 'static,
    {
    }
}

pub struct CallbackNode<F, D, Next> {
    pub(super) deps: D,
    pub(super) callback: F,
    pub(super) next: Next,
}

impl<T, C, F, D, Next> CallbackChain<Inner<T, C>> for CallbackNode<F, D, Next>
where
    T: 'static,
    C: 'static,
    D: ProviderGroup + 'static,
    F: Fn(&mut T, D::Data<'_>) + 'static,
    Next: CallbackChain<Inner<T, C>>,
{
    fn listen<Fg>(&self, weak_src: Weak<Inner<T, C>>, get_node: Fg)
    where
        Fg: Fn(&Inner<T, C>) -> &Self + Copy + 'static,
    {
        let listener = Listener::new(weak_src.clone(), move |src, action| {
            if !action.is_update() {
                return;
            }
            let this = get_node(src);
            (this.callback)(&mut src.value.borrow_mut(), this.deps.read_many());
        });
        self.deps.dependent_many(listener);
        self.next.listen(weak_src, move |src| &get_node(src).next);
    }
}
