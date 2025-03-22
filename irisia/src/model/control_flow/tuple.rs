use crate::{
    model::tools::DirtyPoints,
    prim_element::{EMCreateCtx, Element},
};

use super::{Model, VModel};

impl<A, B> VModel for (A, B)
where
    A: VModel,
    B: VModel,
{
    const EXECUTE_POINTS: usize = A::EXECUTE_POINTS + B::EXECUTE_POINTS;
    type Storage = (A::Storage, B::Storage);
    fn create(self, dp: &mut DirtyPoints, ctx: &EMCreateCtx) -> Self::Storage {
        (self.0.create(dp, ctx), self.1.create(dp, ctx))
    }
    fn update(self, storage: &mut Self::Storage, dp: &mut DirtyPoints, ctx: &EMCreateCtx) {
        self.0.update(&mut storage.0, dp, ctx);
        self.1.update(&mut storage.1, dp, ctx);
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
    const EXECUTE_POINTS: usize = 0;
    type Storage = ();

    fn create(self, _dp: &mut DirtyPoints, _ctx: &EMCreateCtx) -> Self::Storage {}
    fn update(self, _storage: &mut Self::Storage, _dp: &mut DirtyPoints, _ctx: &EMCreateCtx) {}
}

impl Model for () {
    fn visit(&self, _: &mut dyn FnMut(Element)) {}
}
