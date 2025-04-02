use crate::model::{Model, VModel};
use crate::prim_element::{EMCreateCtx, Element};

#[derive(PartialEq)]
pub enum Branch<A, B> {
    A(A),
    B(B),
}

impl<A, B> VModel for Branch<A, B>
where
    A: VModel,
    B: VModel,
{
    type Storage = Branch<A::Storage, B::Storage>;

    fn create(self, ctx: &EMCreateCtx) -> Self::Storage {
        match self {
            Self::A(upd) => {
                let storage = Branch::A(upd.create(ctx));
                storage
            }
            Self::B(upd) => Branch::B(upd.create(ctx)),
        }
    }

    fn update(self, storage: &mut Self::Storage, ctx: &EMCreateCtx) {
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

impl<A, B> Model for Branch<A, B>
where
    A: Model,
    B: Model,
{
    fn visit(&self, f: &mut dyn FnMut(Element)) {
        match self {
            Self::A(a) => a.visit(f),
            Self::B(b) => b.visit(f),
        }
    }
}
