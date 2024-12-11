use crate::el_model::EMCreateCtx;

use super::{
    iter::{ModelMapper, VisitModel},
    VModel,
};

pub enum Branch<A, B> {
    A(A),
    B(B),
}

impl<A, B> VModel for Branch<A, B>
where
    A: VModel,
    B: VModel,
{
    type Storage = BranchModel<A::Storage, B::Storage>;
    fn create(self, ctx: &EMCreateCtx) -> Self::Storage {
        match self {
            Self::A(a) => BranchModel {
                current_is_a: true,
                a: Some(a.create(ctx)),
                b: None,
            },
            Self::B(b) => BranchModel {
                a: None,
                b: Some(b.create(ctx)),
                current_is_a: false,
            },
        }
    }
    fn update(self, storage: &mut Self::Storage, ctx: &EMCreateCtx) {
        match self {
            Self::A(a) => {
                storage.current_is_a = true;
                match &mut storage.a {
                    Some(cache) => a.update(cache, ctx),
                    cache @ None => *cache = Some(a.create(ctx)),
                }
            }
            Self::B(b) => {
                storage.current_is_a = false;
                match &mut storage.b {
                    Some(cache) => b.update(cache, ctx),
                    cache @ None => *cache = Some(b.create(ctx)),
                }
            }
        }
    }
}

pub struct BranchModel<A, B> {
    a: Option<A>,
    b: Option<B>,
    current_is_a: bool,
}

impl<A, B, M> VisitModel<M> for BranchModel<A, B>
where
    A: VisitModel<M>,
    B: VisitModel<M>,
    M: ModelMapper,
{
    fn visit(&self, f: &mut dyn FnMut(<M as ModelMapper>::MapRef<'_>)) {
        if self.current_is_a {
            self.a.as_ref().unwrap().visit(f);
        } else {
            self.b.as_ref().unwrap().visit(f);
        }
    }

    fn visit_mut(&mut self, f: &mut dyn FnMut(<M as ModelMapper>::MapMut<'_>)) {
        if self.current_is_a {
            self.a.as_mut().unwrap().visit_mut(f);
        } else {
            self.b.as_mut().unwrap().visit_mut(f);
        }
    }
}
