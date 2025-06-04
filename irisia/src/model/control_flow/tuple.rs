use crate::{
    model::{Model, ModelCreateCtx, VModel},
    prim_element::Element,
};

impl<A, B, Cd> VModel<Cd> for (A, B)
where
    A: VModel<Cd>,
    B: VModel<Cd>,
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

impl<A, B, Cd> Model<Cd> for (A, B)
where
    A: Model<Cd>,
    B: Model<Cd>,
{
    fn visit(&self, f: &mut dyn FnMut(Element, Cd)) {
        self.0.visit(f);
        self.1.visit(f);
    }
}

impl<Cd> VModel<Cd> for () {
    type Storage = ();

    fn create(&self, _: &ModelCreateCtx) -> Self::Storage {}
    fn update(&self, _: &mut Self::Storage, _: &ModelCreateCtx) {}
}

impl<Cd> Model<Cd> for () {
    fn visit(&self, _: &mut dyn FnMut(Element, Cd)) {}
}
