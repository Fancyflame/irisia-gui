use super::{StructureCreate, VisitBy};
use crate::{data_flow::ReadWire, prim_element::EMCreateCtx};

pub struct Conditional<S1, S2> {
    cond: ReadWire<bool>,
    if_selected: S1,
    or_else: S2,
}

impl<S1, S2, Cp> VisitBy<Cp> for Conditional<S1, S2>
where
    S1: VisitBy<Cp>,
    S2: VisitBy<Cp>,
{
    fn visit<V>(&self, v: &mut V) -> crate::Result<()>
    where
        V: super::Visitor<Cp>,
    {
        if *self.cond.read() {
            self.if_selected.visit(v)
        } else {
            self.or_else.visit(v)
        }
    }

    fn visit_mut<V>(&mut self, v: &mut V) -> crate::Result<()>
    where
        V: super::VisitorMut<Cp>,
    {
        if *self.cond.read() {
            self.if_selected.visit_mut(v)
        } else {
            self.or_else.visit_mut(v)
        }
    }
}

pub fn conditional<Cp, F1, F2>(
    cond: ReadWire<bool>,
    if_selected: F1,
    or_else: F2,
) -> impl StructureCreate<Cp>
where
    F1: StructureCreate<Cp>,
    F2: StructureCreate<Cp>,
{
    move |ctx: &EMCreateCtx| Conditional {
        cond,
        if_selected: if_selected.create(ctx),
        or_else: or_else.create(ctx),
    }
}
