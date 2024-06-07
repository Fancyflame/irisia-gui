use std::cell::RefCell;

use super::{StructureCreate, VisitBy};
use crate::{
    data_flow::{register::register, ReadWire, ReadableExt},
    el_model::EMCreateCtx,
};

enum IfSelected<T, F> {
    Initialized(T),
    Uninitialized(F),
    Intermediate,
}

pub struct Select<T, S1, F1, S2> {
    cond: ReadWire<Option<T>>,
    if_selected: RefCell<IfSelected<S1, F1>>,
    or_else: S2,
}

impl<T, S1, F1, S2> VisitBy for Select<T, S1, F1, S2>
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
                IfSelected::Uninitialized(_) => {
                    drop(borrowed);
                    let mut borrow_mut = self.if_selected.borrow_mut();

                    let IfSelected::Uninitialized(creator) =
                        std::mem::replace(&mut *borrow_mut, IfSelected::Intermediate)
                    else {
                        unreachable!()
                    };

                    let cache_register = register(data.clone());
                    let write_half = cache_register.clone();

                    self.cond.watch(
                        move |cond, _| {
                            if let Some(data) = &*cond.read() {
                                write_half.set(data.clone());
                            }
                        },
                        false,
                    );

                    let tree = creator(cache_register);
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

pub fn branch<T, F1, R1, F2>(
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
    move |ctx: &EMCreateCtx| Select {
        cond,
        if_selected: {
            let ctx = ctx.clone();
            IfSelected::Uninitialized(move |w| if_selected(w).create(&ctx)).into()
        },
        or_else: or_else.create(ctx),
    }
}
