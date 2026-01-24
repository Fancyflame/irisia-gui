use crate::model::{Model, ModelCreateCtx, VModel, VisitModelFn};

impl<A, B> VModel for (A, B)
where
    A: VModel,
    B: VModel,
{
    type Storage = (A::Storage, B::Storage);

    fn create(&self, ctx: &ModelCreateCtx) -> Self::Storage {
        (self.0.create(ctx), self.1.create(ctx))
    }

    fn update(&self, storage: &mut Self::Storage, ctx: &ModelCreateCtx) {
        self.0.update(&mut storage.0, ctx);
        self.1.update(&mut storage.1, ctx);
    }
}

impl<A, B> Model for (A, B)
where
    A: Model,
    B: Model,
{
    fn visit_raw(&self, f: VisitModelFn) {
        self.0.visit_raw(f);
        self.1.visit_raw(f);
    }
}

impl VModel for () {
    type Storage = ();

    fn create(&self, _: &ModelCreateCtx) -> Self::Storage {}
    fn update(&self, _: &mut Self::Storage, _: &ModelCreateCtx) {}
}

impl Model for () {
    fn visit_raw(&self, _: VisitModelFn) {}
}
