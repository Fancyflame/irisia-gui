use std::marker::PhantomData;

use crate::{
    model::{GetParentPropsFn, Model, ModelCreateCtx, VModel},
    prim_element::Element,
};

impl<A, B, Pp> VModel for (A, B)
where
    A: VModel<ParentProps = Pp>,
    B: VModel<ParentProps = Pp>,
{
    type ParentProps = Pp;
    type Storage = (A::Storage, B::Storage);

    fn get_parent_props(&self, f: GetParentPropsFn<Self::ParentProps>) {
        self.0.get_parent_props(f);
        self.1.get_parent_props(f);
    }

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

pub struct Empty<T>(PhantomData<fn() -> T>);

impl<T> Empty<T> {
    pub const fn new() -> Self {
        Empty(PhantomData)
    }
}

impl<T> VModel for Empty<T> {
    type ParentProps = T;
    type Storage = ();

    fn get_parent_props(&self, _: GetParentPropsFn<Self::ParentProps>) {}
    fn create(&self, _ctx: &ModelCreateCtx) -> Self::Storage {}
    fn update(&self, _storage: &mut Self::Storage, _ctx: &ModelCreateCtx) {}
}

impl Model for () {
    fn visit(&self, _: &mut dyn FnMut(Element)) {}
}
