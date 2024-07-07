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

pub struct PatMatch<T, S1, F1, S2> {
    cond: ReadWire<Option<T>>,
    if_selected: RefCell<IfSelected<S1, F1>>,
    or_else: S2,
}

impl<T, S1, F1, S2> VisitBy for PatMatch<T, S1, F1, S2>
where
    T: Clone + 'static,
    F1: FnOnce(ReadWire<T>) -> S1 + 'static,
    S1: VisitBy,
    S2: VisitBy,
{
    fn visit<V>(&self, v: &mut V) -> crate::Result<()>
    where
        V: super::Visitor,
    {
        if let Some(data) = &*self.cond.read() {
            let borrowed = self.if_selected.borrow();
            match &*borrowed {
                IfSelected::Initialized(branch) => branch.visit(v),
                IfSelected::Uninitialized { .. } => {
                    let data = data.clone();
                    drop(borrowed);
                    let mut borrow_mut = self.if_selected.borrow_mut();

                    let IfSelected::Uninitialized(creator) =
                        std::mem::replace(&mut *borrow_mut, IfSelected::Intermediate)
                    else {
                        unreachable!()
                    };

                    let value_wire = {
                        let cond = self.cond.clone();
                        wire3(
                            || {
                                (data, move |mut r| {
                                    if let Some(new_data) = &*cond.read() {
                                        *r = new_data.clone();
                                    }
                                })
                            },
                            true,
                        )
                    };

                    let tree = creator(value_wire);
                    let result = tree.visit(v);
                    *borrow_mut = IfSelected::Initialized(tree);
                    result
                }
                IfSelected::Intermediate => {
                    panic!(
                        "thread was panicked during last updating, this structure should not be used anymore"
                    )
                }
            }
        } else {
            self.or_else.visit(v)
        }
    }
}

pub fn pat_match<T, F1, R1, F2>(
    cond: ReadWire<Option<T>>,
    if_selected: F1,
    or_else: F2,
) -> impl StructureCreate
where
    T: Clone + 'static,
    F1: FnOnce(ReadWire<T>) -> R1 + 'static,
    R1: StructureCreate,
    F2: StructureCreate,
{
    move |ctx: &EMCreateCtx| PatMatch {
        cond,
        if_selected: {
            let ctx = ctx.clone();
            IfSelected::Uninitialized(move |w| if_selected(w).create(&ctx)).into()
        },
        or_else: or_else.create(ctx),
    }
}
