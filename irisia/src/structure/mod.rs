use std::ops::{Deref, DerefMut};

use crate::{
    dep_watch::{bitset::UsizeArray, DependencyIndexes, DependentStack},
    ElModel, Result,
};

pub use self::{
    chain::Chain,
    once::{Once, OnceUpdater},
    repeat::{Repeat, RepeatUpdater},
    select::{Select, SelectUpdateBranch, SelectUpdater},
    slot::{Slot, SlotUpdater},
};

mod bind;
mod chain;
mod empty;
mod once;
mod repeat;
mod select;
mod slot;
pub mod tracert;

pub trait VisitBy {
    fn visit_by<V>(&self, visitor: &mut V) -> Result<()>
    where
        V: VisitOn;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait VisitOn {
    fn visit_on(&mut self, data: &ElModel!(_)) -> Result<()>;
}

pub trait StructureUpdateTo<A: UsizeArray> {
    type Target: VisitBy + 'static;
    const UPDATE_POINTS: u32;

    fn create(self, info: Updating<A>) -> Self::Target;
    fn update(self, target: &mut Self::Target, info: Updating<A>);
}

pub struct Updating<'a, A: UsizeArray> {
    stack: &'a DependentStack<A>,
    update_point_offset: u32,
    points: MaybeBorrowed<'a, A>,
}

enum MaybeBorrowed<'a, A: UsizeArray> {
    Owned(DependencyIndexes<A>),
    Borrowed(&'a mut DependencyIndexes<A>),
}

impl<'a, A: UsizeArray> Updating<'a, A> {
    pub fn new(stack: &'a DependentStack<A>) -> Self {
        Self {
            points: MaybeBorrowed::Owned(stack.get_update_list(true)),
            stack,
            update_point_offset: 0,
        }
    }

    pub fn no_update<T>(&self) -> bool
    where
        T: StructureUpdateTo<A>,
    {
        match self.points.peek() {
            Some(p) if p < self.update_point_offset + T::UPDATE_POINTS => {
                debug_assert!(p >= self.update_point_offset);
                true
            }
            _ => false,
        }
    }

    pub fn scoped<F, R>(&self, relative_position: u32, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.stack
            .scoped(self.update_point_offset + relative_position, f)
    }

    pub fn step_if(&mut self, relative_position: u32) -> bool {
        self.points
            .step_if(self.update_point_offset + relative_position)
    }

    pub(crate) fn inherit(&mut self, relative_position: u32, clone: bool) -> Updating<A> {
        Updating {
            stack: self.stack,
            update_point_offset: self.update_point_offset + relative_position,
            points: if clone {
                MaybeBorrowed::Owned(self.points.clone())
            } else {
                MaybeBorrowed::Borrowed(&mut self.points)
            },
        }
    }
}

impl<A: UsizeArray> Deref for MaybeBorrowed<'_, A> {
    type Target = DependencyIndexes<A>;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(x) => x,
            Self::Owned(x) => x,
        }
    }
}

impl<A: UsizeArray> DerefMut for MaybeBorrowed<'_, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Borrowed(x) => x,
            Self::Owned(x) => x,
        }
    }
}
