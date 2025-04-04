use std::rc::Weak;

use crate::hook::{signal_group::SignalGroup, Listener};

use super::Inner;

pub trait CallbackChain<T> {
    type Storage;

    fn listen<F>(&self, src: Weak<Inner<T>>, get_node: F) -> Self::Storage
    where
        F: Fn(&Inner<T>) -> &Self + Copy + 'static;
}

impl<T> CallbackChain<T> for () {
    type Storage = ();

    fn listen<F>(&self, _: Weak<Inner<T>>, _: F) -> Self::Storage
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
    D: SignalGroup + 'static,
    F: Fn(&mut T, D::Data<'_>) + 'static,
    Next: CallbackChain<T>,
{
    type Storage = CallbackNode<F, (), Next::Storage>;

    fn listen<Fg>(&self, weak_src: Weak<Inner<T>>, get_node: Fg) -> Self::Storage
    where
        Fg: Fn(&Inner<T>) -> &Self + Copy + 'static,
    {
        let listener = Listener::new({
            let weak_src = weak_src.clone();
            move |action| {
                let Some(src) = weak_src.upgrade() else {
                    return false;
                };

                if !action.is_update() {
                    return true;
                }

                let this = get_node(&src);

                if let Some(mut src_mut) = src.value.try_borrow_mut() {
                    (this.callback)(&mut src_mut, D::deref_wrapper(&this.deps.read_many()));
                }

                true
            }
        });
        self.deps.dependent_many(listener);
        self.next.listen(weak_src, move |src| &get_node(src).next);
    }
}
