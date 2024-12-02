use crate::{
    el_model::EMCreateCtx,
    hook::{Consumer, Memo, Provider, ProviderObject, ToProviderObject},
};

use super::{
    iter::{ModelMapper, VisitModel},
    ModelCreateFn,
};

pub struct Optioned<Mat, F, Or>(Consumer<Inner<Mat, F, Or>>);

struct Inner<Mat, F, Or> {
    is_match: bool,
    if_match: MaybeUninitModel<Mat, F>,
    or_else: Or,
    ctx: EMCreateCtx,
}

enum MaybeUninitModel<Model, F> {
    Initialized(Model),
    Uninitialized { create_fn: Option<F> },
}

pub fn optioned<M, P, T, Mat, F, Or>(cond: P, fn_if_match: F, or_else: Or) -> impl ModelCreateFn<M>
where
    T: Clone + 'static,
    M: ModelMapper,
    P: ToProviderObject<Data = Option<T>>,
    F: FnOnce(ProviderObject<T>) -> Mat + Clone + 'static,
    Mat: ModelCreateFn<M>,
    Or: ModelCreateFn<M>,
{
    let provider = cond.to_object();
    move |ctx| {
        let fn_if_match = fn_if_match.clone();
        let mut if_match = MaybeUninitModel::Uninitialized {
            create_fn: Some(|ctx: &_, provider| fn_if_match(provider)(ctx)),
        };

        let is_match = if_match.init_if_matches(ctx, &provider);
        let init_state = Inner {
            is_match,
            if_match,
            or_else: or_else(ctx),
            ctx: ctx.clone(),
        };

        let provider_moved = provider.clone();
        let inner = Consumer::new(
            init_state,
            move |this, _| {
                let is_match = this.if_match.init_if_matches(&this.ctx, &provider_moved);
                if is_match == this.is_match {
                    return;
                }

                this.is_match = is_match;
                this.ctx.model_changed.set(());
            },
            provider.clone(),
        );
        Optioned(inner)
    }
}

impl<Model, F> MaybeUninitModel<Model, F> {
    fn init_if_matches<T>(
        &mut self,
        ctx: &EMCreateCtx,
        provider: &ProviderObject<Option<T>>,
    ) -> bool
    where
        T: Clone + 'static,
        F: FnOnce(&EMCreateCtx, ProviderObject<T>) -> Model,
    {
        let read = provider.read();

        let Some(data) = &*read else {
            return false;
        };

        let create_fn = match self {
            Self::Initialized(_) => return true,
            Self::Uninitialized { create_fn } => create_fn.take().unwrap(),
        };

        let hold_value = Memo::new_customized(
            data.clone(),
            |mut setter, option| {
                if let Some(v) = option {
                    *setter = v.clone();
                }
            },
            provider.clone(),
        )
        .to_object();

        *self = Self::Initialized(create_fn(ctx, hold_value));
        true
    }
}

macro_rules! impl_visit {
    ($visit:ident, $MapRef:ident) => {
        fn $visit(&self, f: &mut dyn FnMut(<M as ModelMapper>::$MapRef<'_>)) {
            let inner = self.0.borrow();
            if inner.is_match {
                match &inner.if_match {
                    MaybeUninitModel::Initialized(init) => init.$visit(f),
                    MaybeUninitModel::Uninitialized { .. } => unreachable!(),
                }
            } else {
                inner.or_else.$visit(f);
            }
        }
    };
}

impl<M, Mat, F, Or> VisitModel<M> for Optioned<Mat, F, Or>
where
    M: ModelMapper,
    Mat: VisitModel<M> + 'static,
    Or: VisitModel<M> + 'static,
    F: 'static,
{
    impl_visit!(visit, MapRef);
    impl_visit!(visit_mut, MapMut);
}
