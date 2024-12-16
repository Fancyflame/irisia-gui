use crate::el_model::EMCreateCtx;

use super::{
    iter::{ModelMapper, VisitModel},
    VModel,
};

impl<A, B> VModel for (A, B)
where
    A: VModel,
    B: VModel,
{
    type Storage = (A::Storage, B::Storage);
    fn create(&self, ctx: &EMCreateCtx) -> Self::Storage {
        (self.0.create(ctx), self.1.create(ctx))
    }
    fn update(&self, storage: &mut Self::Storage, ctx: &EMCreateCtx) {
        self.0.update(&mut storage.0, ctx);
        self.1.update(&mut storage.1, ctx);
    }
}

impl<M, A, B> VisitModel<M> for (A, B)
where
    M: ModelMapper,
    A: VisitModel<M>,
    B: VisitModel<M>,
{
    fn visit(&self, f: &mut dyn FnMut(<M as ModelMapper>::MapRef<'_>)) {
        self.0.visit(f);
        self.1.visit(f);
    }
    fn visit_mut(&mut self, f: &mut dyn FnMut(<M as ModelMapper>::MapMut<'_>)) {
        self.0.visit_mut(f);
        self.1.visit_mut(f);
    }
}

impl VModel for () {
    type Storage = ();
    fn create(&self, _ctx: &EMCreateCtx) -> Self::Storage {}
    fn update(&self, _storage: &mut Self::Storage, _ctx: &EMCreateCtx) {}
}

impl<M> VisitModel<M> for ()
where
    M: ModelMapper,
{
    fn visit(&self, _: &mut dyn FnMut(<M as ModelMapper>::MapRef<'_>)) {}
    fn visit_mut(&mut self, _: &mut dyn FnMut(<M as ModelMapper>::MapMut<'_>)) {}
}
