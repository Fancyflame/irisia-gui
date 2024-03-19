use std::ops::{Deref, DerefMut};

use crate::{
    dep_watch::{
        bitset::U32Array,
        inferer::{BitsetInc, EmptyBitsetInferer},
        DependencyIndexes, DependentStack,
    },
    dom::ElementModel,
    style::StyleSource,
    Result,
};

pub use self::{
    chain::Chain,
    once::{Once, OnceUpdater},
    repeat::{Repeat, RepeatUpdater},
    select::{Select, SelectUpdateBranch, SelectUpdater},
    slot::{Slot, SlotUpdater},
};

macro_rules! bitset_inc {
    ($Ty:ty) => {
        <$Ty as $crate::dep_watch::inferer::BitsetInc>::Result
    };
}

mod bind;
mod chain;
mod empty;
mod once;
mod repeat;
mod select;
mod slot;
pub mod tracert;

type GetBitset<T> = <<T as VisitBy>::AddUpdatePoints<EmptyBitsetInferer> as BitsetInc>::AsBitset;

pub trait VisitBy {
    type AddUpdatePoints<Base: BitsetInc>: BitsetInc;
    const UPDATE_POINTS: u32;

    fn visit_by<V>(&self, visitor: &mut V) -> Result<()>
    where
        V: VisitOn;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait VisitOn {
    fn visit_on<Sty, Slt>(&mut self, data: &ElementModel<Sty, Slt>) -> Result<()>
    where
        Sty: StyleSource;
}

pub trait StructureUpdateTo {
    type Target;

    fn update(this: &mut Self::Target, upd: Updating<impl U32Array>);
    fn create(upd: Updating<impl U32Array>) -> Self::Target;
}

pub struct Updating<'a, A>
where
    A: U32Array,
{
    call_stack: &'a DependentStack<A>,
    update_point_offset: u32,
    points: MaybeBorrowed<'a, A>,
}

enum MaybeBorrowed<'a, A: U32Array> {
    Owned(DependencyIndexes<A>),
    Borrowed(&'a mut DependencyIndexes<A>),
}

impl<'a, A: U32Array> Updating<'a, A> {
    pub fn new(stack: &'a DependentStack<A>) -> Self {
        Self {
            points: MaybeBorrowed::Owned(stack.get_update_list(true)),
            call_stack: stack,
            update_point_offset: 0,
        }
    }

    pub fn no_update<T>(&self) -> bool
    where
        T: VisitBy,
    {
        match self.points.peek() {
            Some(p) if p < self.update_point_offset + T::UPDATE_POINTS => {
                debug_assert!(p >= self.update_point_offset);
                true
            }
            _ => false,
        }
    }

    pub fn union_offseted(&mut self, with: impl U32Array) {
        let mut buffer = 0u32;
        let chunk_offset = (self.update_point_offset / 32) as usize;
        let bit_offset = self.update_point_offset % 32;

        for (index, &x) in with.as_ref().iter().enumerate() {
            buffer |= x << bit_offset;
            self.points.bitset[chunk_offset + index] |= buffer;
            buffer = x >> (32 - bit_offset);
        }

        if buffer != 0 {
            self.points.bitset[chunk_offset + with.as_ref().len()] |= buffer;
        }
    }

    pub fn scoped<F, R>(&self, relative_position: u32, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.call_stack
            .scoped(self.update_point_offset + relative_position, f)
    }

    pub fn step_if(&mut self, relative_position: u32) -> bool {
        self.points
            .step_if(self.update_point_offset + relative_position)
    }

    pub(crate) fn inherit(&mut self, relative_position: u32, clone: bool) -> Updating<A> {
        Updating {
            call_stack: self.call_stack,
            update_point_offset: self.update_point_offset + relative_position,
            points: if clone {
                MaybeBorrowed::Owned(self.points.clone())
            } else {
                MaybeBorrowed::Borrowed(&mut self.points)
            },
        }
    }
}

impl<A: U32Array> Deref for MaybeBorrowed<'_, A> {
    type Target = DependencyIndexes<A>;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(x) => x,
            Self::Owned(x) => x,
        }
    }
}

impl<A: U32Array> DerefMut for MaybeBorrowed<'_, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Borrowed(x) => x,
            Self::Owned(x) => x,
        }
    }
}
