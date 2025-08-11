use crate::model::{EleModel, Model, ModelCreateCtx, VModel};
use crate::prim_element::Element;

#[derive(PartialEq)]
pub enum Branch<A, B> {
    A(A),
    B(B),
}

impl<T, Cd> VModel<Cd> for Option<T>
where
    T: VModel<Cd>,
{
    type Storage = Branch<T::Storage, ()>;

    fn create(&self, ctx: &ModelCreateCtx) -> Self::Storage {
        match self {
            Some(v) => Branch::A(v),
            None => Branch::B(()),
        }
        .create(ctx)
    }

    fn update(&self, storage: &mut Self::Storage, ctx: &ModelCreateCtx) {
        match self {
            Some(v) => Branch::A(v),
            None => Branch::B(()),
        }
        .update(storage, ctx);
    }
}

impl<A, B, Cd> VModel<Cd> for Branch<A, B>
where
    A: VModel<Cd>,
    B: VModel<Cd>,
{
    type Storage = Branch<A::Storage, B::Storage>;

    fn create(&self, ctx: &ModelCreateCtx) -> Self::Storage {
        match self {
            Self::A(upd) => {
                let storage = Branch::A(upd.create(ctx));
                storage
            }
            Self::B(upd) => Branch::B(upd.create(ctx)),
        }
    }

    fn update(&self, storage: &mut Self::Storage, ctx: &ModelCreateCtx) {
        match self {
            Self::A(upd) => {
                if let Branch::A(cache) = storage {
                    upd.update(cache, ctx);
                } else {
                    *storage = Branch::A(upd.create(ctx));
                }
            }
            Self::B(upd) => {
                if let Branch::B(cache) = storage {
                    upd.update(cache, ctx);
                } else {
                    *storage = Branch::B(upd.create(ctx));
                }
            }
        }
    }
}

impl<A, B, Cd> Model<Cd> for Branch<A, B>
where
    A: Model<Cd>,
    B: Model<Cd>,
{
    fn visit(&self, f: &mut dyn FnMut(Element, Cd)) {
        match self {
            Self::A(a) => a.visit(f),
            Self::B(b) => b.visit(f),
        }
    }
}

impl<A, B, Cd> EleModel<Cd> for Branch<A, B>
where
    A: EleModel<Cd>,
    B: EleModel<Cd>,
{
    fn get_element(&self) -> (Element, Cd) {
        match self {
            Self::A(a) => a.get_element(),
            Self::B(b) => b.get_element(),
        }
    }
}
