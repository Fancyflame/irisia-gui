use std::{cell::Cell, ops::Deref};

use crate::dep_watch::{bitset::U32Array, Bitset, DependentStack};

pub fn map_trace<A, T, Tup, F>(tracert: Tracert<T, A>, f: F)
where
    A: U32Array,
    F: FnOnce(T) -> Tup,
    Tup: TupleWatch,
{
    f(tracert.data).trace_all(tracert.base)
}

pub struct Tracert<'a, T, A: U32Array> {
    data: T,
    base: TracertBase<'a, A>,
}

#[derive(Clone, Copy)]
pub struct TracertBase<'a, A: U32Array> {
    pub(crate) dep_stack: &'a DependentStack<A>,
    pub(crate) bitset: &'a Cell<Bitset<A>>,
}

pub trait TupleWatch {
    type Output<'a, A: U32Array>;
    fn trace_all<'a, A: U32Array>(self, tb: TracertBase<'a, A>) -> Self::Output<'a, A>;
}

impl<T, A: U32Array> Deref for Tracert<'_, T, A> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.base
            .bitset
            .set(self.base.dep_stack.collect_dep(self.base.bitset.get()));
        &self.data
    }
}

impl<'a, A: U32Array> TracertBase<'a, A> {
    pub(crate) fn new(dep_stack: &'a DependentStack<A>, bitset: &'a Cell<Bitset<A>>) -> Self {
        Self { dep_stack, bitset }
    }
}

macro_rules! trace_tuple {
    ()=>{};
    ($T0:ident $var0:ident $($T:ident $var:ident)*) => {
        impl<$($T),*> TupleWatch for ($($T,)*) {
            type Output<'a, A: U32Array> = ($(Tracert<'a, $T, A>,)*);

            fn trace_all<'a, A: U32Array>(self, _tb: TracertBase<'a, A>) -> Self::Output<'a, A> {
                let ($($var,)*) = self;
                ($(_tb.into_tracert($var),)*)
            }
        }

        trace_tuple!($($T $var)*);
    };
}

trace_tuple! {
    A a
    B b C c D d E e F f
    G g H h I i J j K k
    L l M m N n O o P p
    Q q R r S s T t U u
    V v W w X x Y y Z z
    Aa aa Bb bb Cc cc Dd dd Ee ee
    Ff ff Gg gg
}
