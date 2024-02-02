use std::ops::{Deref, DerefMut};

use crate::{
    ElModel, Result,
    __private::dep_stack::{DependencyIndexes, DependentStack},
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

pub trait StructureUpdateTo<const WD: usize> {
    type Target;
    const UPDATE_POINTS: u32;

    fn create(self, info: Updating<WD>) -> Self::Target;
    fn update(self, target: &mut Self::Target, info: Updating<WD>);
}

pub struct Updating<'a, const WD: usize> {
    stack: &'a DependentStack<WD>,
    update_point_offset: u32,
    points: MaybeBorrowed<'a, WD>,
}

enum MaybeBorrowed<'a, const WD: usize> {
    Owned(DependencyIndexes<WD>),
    Borrowed(&'a mut DependencyIndexes<WD>),
}

impl<'a, const WD: usize> Updating<'a, WD> {
    pub fn new(stack: &'a DependentStack<WD>, dep_idx: DependencyIndexes<WD>) -> Self {
        Self {
            stack,
            update_point_offset: 0,
            points: MaybeBorrowed::Owned(dep_idx),
        }
    }

    pub fn no_update<T>(&self) -> bool
    where
        T: StructureUpdateTo<WD>,
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

    pub(crate) fn inherit(&mut self, relative_position: u32, clone: bool) -> Updating<WD> {
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

impl<const WD: usize> Deref for MaybeBorrowed<'_, WD> {
    type Target = DependencyIndexes<WD>;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(x) => x,
            Self::Owned(x) => x,
        }
    }
}

impl<const WD: usize> DerefMut for MaybeBorrowed<'_, WD> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Borrowed(x) => x,
            Self::Owned(x) => x,
        }
    }
}
