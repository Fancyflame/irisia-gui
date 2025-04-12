use crate::{
    model::{Model, ModelCreateCtx, VModel},
    prim_element::Element,
};

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
    fn visit(&self, f: &mut dyn FnMut(Element)) {
        self.0.visit(f);
        self.1.visit(f);
    }
}

impl VModel for () {
    type Storage = ();

    fn create(&self, _ctx: &ModelCreateCtx) -> Self::Storage {}
    fn update(&self, _storage: &mut Self::Storage, _ctx: &ModelCreateCtx) {}
}

impl Model for () {
    fn visit(&self, _: &mut dyn FnMut(Element)) {}
}
