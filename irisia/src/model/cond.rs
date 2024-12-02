use crate::{
    hook::{Consumer, Provider, State, ToProviderObject},
    model::{iter::ModelMapper, ModelCreateFn},
};

use super::iter::VisitModel;

pub struct Cond<Tr, Fa>(Consumer<Inner<Tr, Fa>>);

struct Inner<Tr, Fa> {
    cond: bool,
    if_true: Tr,
    if_false: Fa,
    model_change: State<()>,
}

pub fn cond<T, M, Tr, Fa>(cond: T, if_true: Tr, if_false: Fa) -> impl ModelCreateFn<M>
where
    T: ToProviderObject<Data = bool>,
    M: ModelMapper,
    Tr: ModelCreateFn<M>,
    Fa: ModelCreateFn<M>,
{
    let cond = cond.to_object();
    move |ctx| {
        let init_state = Inner {
            cond: *cond.read(),
            if_true: if_true(ctx),
            if_false: if_false(ctx),
            model_change: ctx.model_changed.clone(),
        };

        let inner = Consumer::builder(init_state)
            .dep(
                |this, &cond| {
                    if this.cond != cond {
                        this.cond = cond;
                        this.model_change.set(());
                    }
                },
                cond.clone(),
            )
            .build();
        Cond(inner)
    }
}

impl<M: ModelMapper, Tr, Fa> VisitModel<M> for Cond<Tr, Fa>
where
    Tr: VisitModel<M> + 'static,
    Fa: VisitModel<M> + 'static,
{
    fn visit(&self, f: &mut dyn FnMut(<M as ModelMapper>::MapRef<'_>)) {
        let inner = self.0.borrow();
        if inner.cond {
            inner.if_true.visit(f);
        } else {
            inner.if_false.visit(f);
        }
    }

    fn visit_mut(&self, f: &mut dyn FnMut(<M as ModelMapper>::MapMut<'_>)) {
        let inner = self.0.borrow_mut();
        if inner.cond {
            inner.if_true.visit_mut(f);
        } else {
            inner.if_false.visit_mut(f);
        }
    }
}
