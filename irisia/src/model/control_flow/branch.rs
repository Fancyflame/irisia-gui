use crate::model::{EleModel, GetParentPropsFn, Model, ModelCreateCtx, VModel};
use crate::prim_element::Element;

use super::miscellaneous::Empty;

pub fn branch_a<A, B>(value: A) -> Branch<A, B>
where
    A: VModel,
    B: VModel,
{
    Branch::A(value)
}

pub fn branch_b<A, B>(value: B) -> Branch<A, B>
where
    A: VModel,
    B: VModel,
{
    Branch::B(value)
}

#[derive(PartialEq)]
pub enum Branch<A, B> {
    A(A),
    B(B),
}

impl<T> VModel for Option<T>
where
    T: VModel,
{
    type Storage = Branch<T::Storage, ()>;
    type ParentProps = T::ParentProps;

    fn get_parent_props(&self, f: GetParentPropsFn<Self::ParentProps>) {
        if let Some(value) = self {
            value.get_parent_props(f);
        }
    }

    fn create(&self, ctx: &ModelCreateCtx) -> Self::Storage {
        match self {
            Some(v) => Branch::A(v),
            None => Branch::B(Empty::new()),
        }
        .create(ctx)
    }

    fn update(&self, storage: &mut Self::Storage, ctx: &ModelCreateCtx) {
        match self {
            Some(v) => Branch::A(v),
            None => Branch::B(Empty::new()),
        }
        .update(storage, ctx);
    }
}

impl<A, B, Pp> VModel for Branch<A, B>
where
    A: VModel<ParentProps = Pp>,
    B: VModel<ParentProps = Pp>,
{
    type Storage = Branch<A::Storage, B::Storage>;
    type ParentProps = Pp;

    fn get_parent_props(&self, f: GetParentPropsFn<Pp>) {
        match self {
            Self::A(a) => a.get_parent_props(f),
            Self::B(b) => b.get_parent_props(f),
        }
    }

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

impl<A, B> EleModel for Branch<A, B>
where
    A: EleModel,
    B: EleModel,
{
    fn get_element(&self) -> Element {
        match self {
            Self::A(a) => a.get_element(),
            Self::B(b) => b.get_element(),
        }
    }
}
