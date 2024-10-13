use std::cell::RefCell;

use super::{StructureCreate, VisitBy};
use crate::{
    data_flow::{wire3, ReadWire},
    el_model::EMCreateCtx,
};

enum IfSelected<T, F> {
    Initialized(T),
    Uninitialized(F),
    Intermediate,
}

pub struct PatMatch<Src, T, S1, F1, S2> {
    src: ReadWire<Src>,
    judge_fn: fn(&Src) -> Option<&T>,
    if_selected: RefCell<IfSelected<S1, F1>>,
    or_else: S2,
}

impl<Src, T, S1, F1, S2> PatMatch<Src, T, S1, F1, S2>
where
    Src: 'static,
    T: Clone + 'static,
    F1: FnOnce(ReadWire<T>) -> S1,
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

        let value_wire = {
            let src = self.src.clone();
            let judge_fn = self.judge_fn;
            wire3(move || {
                let init_state = judge_fn(&src.read()).unwrap().clone();
                (init_state, move |mut setter| {
                    if let Some(value) = judge_fn(&src.read()) {
                        *setter = value.clone();
                    }
                })
            })
        };

        *borrow_mut = IfSelected::Initialized(creator(value_wire));
    }
}

impl<Cp, Src, T, S1, F1, S2> VisitBy<Cp> for PatMatch<Src, T, S1, F1, S2>
where
    Src: 'static,
    T: Clone + 'static,
    F1: FnOnce(ReadWire<T>) -> S1 + 'static,
    S1: VisitBy<Cp>,
    S2: VisitBy<Cp>,
{
    fn visit<V>(&self, v: &mut V) -> crate::Result<()>
    where
        V: super::Visitor<Cp>,
    {
        if (self.judge_fn)(&*self.src.read()).is_none() {
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
        if (self.judge_fn)(&*self.src.read()).is_none() {
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

pub fn pat_match<Src, T, F1, R1, R2>(
    src: ReadWire<Src>,
    cond: fn(&Src) -> Option<&T>,
    if_selected: F1,
    or_else: R2,
) -> impl StructureCreate
where
    T: Clone + 'static,
    F1: FnOnce(ReadWire<T>) -> R1 + 'static,
    R1: StructureCreate,
    R2: StructureCreate,
{
    move |ctx: &EMCreateCtx| PatMatch {
        src,
        judge_fn: cond,
        if_selected: {
            let ctx = ctx.clone();
            IfSelected::Uninitialized::<T, _>(move |w| if_selected(w).create(&ctx)).into()
        },
        or_else: or_else.create(ctx),
    }
}
