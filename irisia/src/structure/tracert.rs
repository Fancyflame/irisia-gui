use std::{cell::Cell, ops::Deref};

use crate::__private::dep_stack::{Bitset, DependentStack};

pub struct Tracert<'a, T, const WD: usize> {
    data: T,
    base: TracertBase<'a, WD>,
}

#[derive(Clone, Copy)]
pub struct TracertBase<'a, const WD: usize> {
    pub(crate) dep_stack: &'a DependentStack<WD>,
    pub(crate) bitset: &'a Cell<Bitset<WD>>,
}

pub trait TupleWatch {
    type Output<'a, const WD: usize>;
    fn trace_all<'a, const WD: usize>(self, tb: TracertBase<'a, WD>) -> Self::Output<'a, WD>;
}

impl<T, const WD: usize> Deref for Tracert<'_, T, WD> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.base
            .bitset
            .set(self.base.dep_stack.collect_dep(self.base.bitset.get()));
        &self.data
    }
}

impl<'a, const WD: usize> TracertBase<'a, WD> {
    pub(crate) fn new(dep_stack: &'a DependentStack<WD>, bitset: &'a Cell<Bitset<WD>>) -> Self {
        Self { dep_stack, bitset }
    }

    pub fn tuple_into_tracert<T: TupleWatch>(self, tuple: T) -> T::Output<'a, WD> {
        tuple.trace_all(self)
    }

    pub fn into_tracert<T>(self, data: T) -> Tracert<'a, T, WD> {
        Tracert { data, base: self }
    }
}

macro_rules! trace_tuple {
    ()=>{};
    ($T0:ident $var0:ident $($T:ident $var:ident)*) => {
        impl<$($T),*> TupleWatch for ($($T,)*) {
            type Output<'a, const WD: usize> = ($(Tracert<'a, $T, WD>,)*);

            fn trace_all<'a, const WD: usize>(self, _tb: TracertBase<'a, WD>) -> Self::Output<'a, WD> {
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
