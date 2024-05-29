use super::{StructureCreate, VisitBy};
use crate::{data_flow::ReadWire, el_model::EMCreateCtx};

pub struct Select<T, U> {
    cond: ReadWire<bool>,
    if_selected: T,
    or_else: U,
}

impl<T, U> VisitBy for Select<T, U>
where
    T: VisitBy,
    U: VisitBy,
{
    fn visit<V>(&self, v: &mut V) -> crate::Result<()>
    where
        V: super::Visitor,
    {
        if *self.cond.read() {
            self.if_selected.visit(v)
        } else {
            self.or_else.visit(v)
        }
    }

    fn len(&self) -> usize {
        if *self.cond.read() {
            self.if_selected.len()
        } else {
            self.or_else.len()
        }
    }
}

pub fn branch<F1, F2>(cond: ReadWire<bool>, if_selected: F1, or_else: F2) -> impl StructureCreate
where
    F1: StructureCreate,
    F2: StructureCreate,
{
    move |ctx: &EMCreateCtx| Select {
        cond,
        if_selected: if_selected.create(ctx),
        or_else: or_else.create(ctx),
    }
}
