use anyhow::anyhow;

use crate::{
    structure::{VisitBy, VisitOn},
    Result,
};

pub trait SelectVisitBy {
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
