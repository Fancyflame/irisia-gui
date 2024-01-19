use anyhow::anyhow;

use crate::{
    structure::{Slot, VisitBy, VisitOn},
    Result,
};

pub trait SelectVisitBy {
    type ExtendNode<T: VisitBy, Slt>: SelectVisitBy;

    fn extend<T: VisitBy, Slt>(self, slot: Slot<Slt>) -> Self::ExtendNode<T, Slt>;
    fn visit<V: VisitOn>(&self, index: usize, visitor: &mut V) -> Result<()>;
    fn len(&self, index: usize) -> usize;
}

pub struct SelectBody<T, B, Slt> {
    pub data: BranchData<T, Slt>,
    pub trailing: B,
}

pub enum BranchData<T, Slt> {
    Initialized(T),
    Uninitialized(Slot<Slt>),
}

impl<T, B, Slt> SelectVisitBy for SelectBody<T, B, Slt>
where
    T: VisitBy,
    B: SelectVisitBy,
{
    type ExtendNode<U: VisitBy, Slt2> = SelectBody<T, B::ExtendNode<U, Slt2>, Slt>;

    fn extend<U: VisitBy, Slt2>(self, slot: Slot<Slt2>) -> Self::ExtendNode<U, Slt2> {
        SelectBody {
            data: self.data,
            trailing: self.trailing.extend(slot),
        }
    }

    fn visit<V: VisitOn>(&self, index: usize, visitor: &mut V) -> crate::Result<()> {
        match index.checked_sub(1) {
            Some(new_index) => self.trailing.visit(new_index, visitor),
            None => match &self.data {
                BranchData::Initialized(this) => this.visit_by(visitor),
                BranchData::Uninitialized(_) => {
                    panic!("attempt to select an uninitialized select-body");
                }
            },
        }
    }

    fn len(&self, index: usize) -> usize {
        match index.checked_sub(1) {
            Some(new_index) => self.trailing.len(new_index),
            None => match &self.data {
                BranchData::Initialized(this) => this.len(),
                BranchData::Uninitialized(_) => {
                    panic!("attempt to select an uninitialized select-body")
                }
            },
        }
    }
}

const EOC_ERR: &str = "reached end of select chain";

impl SelectVisitBy for () {
    type ExtendNode<T: VisitBy, Slt> = SelectBody<T, (), Slt>;

    fn extend<T: VisitBy, Slt>(self, slot: Slot<Slt>) -> Self::ExtendNode<T, Slt> {
        SelectBody {
            data: BranchData::Uninitialized(slot),
            trailing: (),
        }
    }

    fn visit<V: VisitOn>(&self, _: usize, _: &mut V) -> Result<()> {
        if cfg!(debug_assertions) {
            unreachable!("{EOC_ERR}");
        } else {
            Err(anyhow!("{EOC_ERR}"))
        }
    }

    fn len(&self, _: usize) -> usize {
        if cfg!(debug_assertions) {
            unreachable!("{EOC_ERR}");
        } else {
            0
        }
    }
}
