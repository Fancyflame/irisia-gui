use crate::{
    model::tools::DirtyPoints,
    prim_element::{EMCreateCtx, Element},
};

use super::{Model, VModel};

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
    const EXECUTE_POINTS: usize = A::EXECUTE_POINTS + B::EXECUTE_POINTS;
    type Storage = Branch<A::Storage, B::Storage>;

    fn create(self, dp: &mut DirtyPoints, ctx: &EMCreateCtx) -> Self::Storage {
        match self {
            Self::A(upd) => {
                let storage = Branch::A(upd.create(dp, ctx));
                dp.consume(B::EXECUTE_POINTS);
                storage
            }
            Self::B(upd) => {
                dp.consume(A::EXECUTE_POINTS);
                Branch::B(upd.create(dp, ctx))
            }
        }
    }

    fn update(self, storage: &mut Self::Storage, dp: &mut DirtyPoints, ctx: &EMCreateCtx) {
        match self {
            Self::A(upd) => {
                if let Branch::A(cache) = storage {
                    upd.update(cache, dp, ctx);
                } else {
                    *storage = Branch::A(upd.create(dp, ctx));
                }
                dp.consume(B::EXECUTE_POINTS);
            }
            Self::B(upd) => {
                dp.consume(A::EXECUTE_POINTS);
                if let Branch::B(cache) = storage {
                    upd.update(cache, dp, ctx);
                } else {
                    *storage = Branch::B(upd.create(dp, ctx));
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
