use anyhow::anyhow;

use crate::{
    structure::{UpdateNode, UpdateSlot, UpdateSlotFn, VisitBy, VisitOn},
    Result,
};

pub trait SelectVisitBy {
    type ExtendNode<T: VisitBy>: SelectVisitBy;

    fn extend<T: VisitBy>(self) -> Self::ExtendNode<T>;
    fn visit<V: VisitOn>(&self, index: usize, visitor: &mut V) -> Result<()>;
    fn len(&self, index: usize) -> usize;
}

pub struct SelectBody<T, B> {
    pub this: Option<T>,
    pub trailing: B,
}

impl<T, B> SelectVisitBy for SelectBody<T, B>
where
    T: VisitBy,
    B: SelectVisitBy,
{
    type ExtendNode<U: VisitBy> = SelectBody<T, B::ExtendNode<U>>;

    fn extend<U: VisitBy>(self) -> Self::ExtendNode<U> {
        SelectBody {
            this: self.this,
            trailing: self.trailing.extend(),
        }
    }

    fn visit<V: VisitOn>(&self, index: usize, visitor: &mut V) -> crate::Result<()> {
        match index.checked_sub(1) {
            Some(new_index) => self.trailing.visit(new_index, visitor),
            None => match &self.this {
                Some(this) => this.visit_by(visitor),
                None => {
                    panic!("attempt to select an uninitialized select-body");
                }
            },
        }
    }

    fn len(&self, index: usize) -> usize {
        match index.checked_sub(1) {
            Some(new_index) => self.trailing.len(new_index),
            None => self
                .this
                .as_ref()
                .map(T::len)
                .expect("attempt to select an uninitialized select-body"),
        }
    }
}

const EOC_ERR: &str = "reached end of select chain";

impl SelectVisitBy for () {
    type ExtendNode<T: VisitBy> = SelectBody<T, ()>;

    fn extend<T: VisitBy>(self) -> Self::ExtendNode<T> {
        SelectBody {
            this: None,
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

impl<Slt, T, B> UpdateSlot<Slt> for SelectBody<T, B>
where
    T: UpdateSlot<Slt>,
    B: UpdateSlot<Slt>,
{
    fn will_update() -> bool {
        T::will_update() || B::will_update()
    }

    fn update_slot(&mut self, f: UpdateSlotFn<Slt>) {
        if T::will_update() {
            match &mut self.this {
                Some(this) => f(UpdateNode::NeedsUpdate(this)),
                this @ None => f(UpdateNode::NeedsInit(this)),
            }
        } else if B::will_update() {
            self.trailing.update_slot(f);
        }
    }
}
