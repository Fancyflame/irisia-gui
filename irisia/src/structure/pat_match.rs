use std::cell::RefCell;

use super::{StructureCreate, VisitBy};
use crate::{
    data_flow::{wire, wire3, ReadWire, ToReadWire},
    prim_element::EMCreateCtx,
};

enum IfSelected<S, F> {
    Initialized(S),
    Uninitialized(F),
    Intermediate,
}

pub struct PatMatch<T, S1, F1, S2> {
    cond: ReadWire<Option<T>>,
    if_selected: RefCell<IfSelected<S1, F1>>,
    or_else: S2,
}

impl<T, S1, F1, S2> PatMatch<T, S1, F1, S2>
where
    F1: FnOnce() -> S1,
{
    fn init_if_need(&self) {
        let borrowed = self.if_selected.borrow();
        match &*borrowed {
            IfSelected::Initialized(_) => return,
            IfSelected::Intermediate => panic!(
                "thread was panicked during last updating, this structure should not be used anymore"
            ),
            IfSelected::Uninitialized(_) => {}
        }

        drop(borrowed);
        let mut borrow_mut = self.if_selected.borrow_mut();

        let IfSelected::Uninitialized(creator) =
            std::mem::replace(&mut *borrow_mut, IfSelected::Intermediate)
        else {
            unreachable!()
        };

        *borrow_mut = IfSelected::Initialized(creator());
    }
}

impl<Cp, T, S1, F1, S2> VisitBy<Cp> for PatMatch<T, S1, F1, S2>
where
    T: Clone + 'static,
    F1: FnOnce() -> S1 + 'static,
    S1: VisitBy<Cp>,
    S2: VisitBy<Cp>,
{
    fn visit<V>(&self, v: &mut V) -> crate::Result<()>
    where
        V: super::Visitor<Cp>,
    {
        if self.cond.read().is_none() {
            return self.or_else.visit(v);
        }

        self.init_if_need();
        if let IfSelected::Initialized(x) = &*self.if_selected.borrow() {
            x.visit(v)
        } else {
            unreachable!();
        }
    }

    fn visit_mut<V>(&mut self, v: &mut V) -> crate::Result<()>
    where
        V: super::VisitorMut<Cp>,
    {
        if self.cond.read().is_none() {
            return self.or_else.visit_mut(v);
        }

        self.init_if_need();
        if let IfSelected::Initialized(x) = &mut *self.if_selected.borrow_mut() {
            x.visit_mut(v)
        } else {
            unreachable!();
        }
    }
}

pub fn pat_match<Cp, ToSrc, Src, T, F1, R1, R2>(
    src: ToSrc,
    cond: fn(&Src) -> Option<T>,
    if_selected: F1,
    or_else: R2,
) -> impl StructureCreate<Cp>
where
    ToSrc: ToReadWire<Data = Src>,
    Src: 'static,
    T: Clone + 'static,
    F1: FnOnce(ReadWire<T>) -> R1 + 'static,
    R1: StructureCreate<Cp>,
    R2: StructureCreate<Cp>,
{
    let src = src.to_read_wire();
    let opt_wire = wire(move |src| cond(&src.read()), src.clone());
    move |ctx: &EMCreateCtx| {
        let creator = {
            let ctx = ctx.clone();
            let opt_wire = opt_wire.clone();
            move || {
                let w = wire3(move || {
                    let init_state = opt_wire.read().as_ref().unwrap().clone();
                    (init_state, move |mut setter| {
                        if let Some(value) = &*opt_wire.read() {
                            *setter = value.clone();
                        }
                    })
                });
                if_selected(w).create(&ctx)
            }
        };

        PatMatch {
            cond: opt_wire.clone(),
            if_selected: IfSelected::Uninitialized::<R1::Target, _>(creator).into(),
            or_else: or_else.create(ctx),
        }
    }
}
