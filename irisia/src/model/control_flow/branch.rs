use crate::{prim_element::EMCreateCtx, prim_element::Element};

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
    type Storage = BranchModel<A::Storage, B::Storage>;
    fn create(self, ctx: &EMCreateCtx) -> Self::Storage {
        match self {
            Self::A(a) => BranchModel {
                current_is_a: true,
                a: Some(a.create(ctx)),
                b: None,
            },
            Self::B(b) => BranchModel {
                current_is_a: false,
                a: None,
                b: Some(b.create(ctx)),
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

impl<A, B> Model for BranchModel<A, B>
where
    A: Model,
    B: Model,
{
    fn visit(&self, f: &mut dyn FnMut(Element)) {
        if self.current_is_a {
            self.a.as_ref().unwrap().visit(f);
        } else {
            self.b.as_ref().unwrap().visit(f);
        }
    }
}
