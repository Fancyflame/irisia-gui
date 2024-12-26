use crate::{el_model::EMCreateCtx, prim_element::Element};

use super::{Model, VModel};

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
    fn create(&self, _ctx: &EMCreateCtx) -> Self::Storage {}
    fn update(&self, _storage: &mut Self::Storage, _ctx: &EMCreateCtx) {}
}

impl Model for () {
    fn visit(&self, _: &mut dyn FnMut(Element)) {}
}
