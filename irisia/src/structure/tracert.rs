use std::{cell::Cell, ops::Deref};

use crate::dep_watch::{bitset::UsizeArray, Bitset, DependentStack};

pub struct Tracert<'a, T, A: UsizeArray> {
    data: T,
    base: TracertBase<'a, A>,
}

#[derive(Clone, Copy)]
pub struct TracertBase<'a, A: UsizeArray> {
    pub(crate) dep_stack: &'a DependentStack<A>,
    pub(crate) bitset: &'a Cell<Bitset<A>>,
}

pub trait TupleWatch {
    type Output<'a, A: UsizeArray>;
    fn trace_all<'a, A: UsizeArray>(self, tb: TracertBase<'a, A>) -> Self::Output<'a, A>;
}

impl<T, A: UsizeArray> Deref for Tracert<'_, T, A> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.base
            .bitset
            .set(self.base.dep_stack.collect_dep(self.base.bitset.get()));
        &self.data
    }
}

impl<'a, A: UsizeArray> TracertBase<'a, A> {
    pub(crate) fn new(dep_stack: &'a DependentStack<A>, bitset: &'a Cell<Bitset<A>>) -> Self {
        Self { dep_stack, bitset }
    }

    pub fn tuple_into_tracert<T: TupleWatch>(self, tuple: T) -> T::Output<'a, A> {
        tuple.trace_all(self)
    }

    pub fn into_tracert<T>(self, data: T) -> Tracert<'a, T, A> {
        Tracert { data, base: self }
    }
}

macro_rules! trace_tuple {
    ()=>{};
    ($T0:ident $var0:ident $($T:ident $var:ident)*) => {
        impl<$($T),*> TupleWatch for ($($T,)*) {
            type Output<'a, A: UsizeArray> = ($(Tracert<'a, $T, A>,)*);

            fn trace_all<'a, A: UsizeArray>(self, _tb: TracertBase<'a, A>) -> Self::Output<'a, A> {
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
