use crate::{
    model::optional_update::DirtyPoints,
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

    fn create(self, exec_point_offset: usize, ctx: &EMCreateCtx) -> Self::Storage {
        match self {
            Self::A(upd) => Branch::A(upd.create(exec_point_offset, ctx)),
            Self::B(upd) => Branch::B(upd.create(exec_point_offset, ctx)),
        }
    }

    fn update(self, storage: &mut Self::Storage, dp: DirtyPoints, ctx: &EMCreateCtx) {
        match self {
            Self::A(upd) => {
                if let Branch::A(cache) = storage {
                    upd.update(cache, dp, ctx);
                } else {
                    *storage = Branch::A(upd.create(dp.offset(), ctx));
                }
            }
            Self::B(upd) => {
                if let Branch::B(cache) = storage {
                    upd.update(cache, dp, ctx);
                } else {
                    *storage = Branch::B(upd.create(dp.offset() + A::EXECUTE_POINTS, ctx));
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
