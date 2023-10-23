use crate::{
    structure::{VisitBy, VisitLen, VisitMutBy},
    Result,
};

pub trait SelectVisitLen {
    fn len(&self, index: usize) -> usize;
}

pub trait SelectVisitBy<V>: SelectVisitLen {
    fn visit(&self, index: usize, visitor: &mut V) -> Result<()>;
}

pub trait SelectVisitMutBy<V>: SelectVisitLen {
    fn visit_mut(&mut self, index: usize, visitor: &mut V) -> Result<()>;
}

pub struct SelectBody<T, B> {
    pub this: Option<T>,
    pub trailing: B,
}

impl<T, B> SelectVisitLen for SelectBody<T, B>
where
    T: VisitLen,
    B: SelectVisitLen,
{
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

impl<T, B, V> SelectVisitBy<V> for SelectBody<T, B>
where
    T: VisitBy<V>,
    B: SelectVisitBy<V>,
{
    fn visit(&self, index: usize, visitor: &mut V) -> crate::Result<()> {
        match index.checked_sub(1) {
            Some(new_index) => self.trailing.visit(new_index, visitor),
            None => match &self.this {
                Some(this) => this.visit(visitor),
                None => {
                    panic!("attempt to select an uninitialized select-body");
                }
            },
        }
    }
}

impl<T, B, V> SelectVisitMutBy<V> for SelectBody<T, B>
where
    T: VisitMutBy<V>,
    B: SelectVisitMutBy<V>,
{
    fn visit_mut(&mut self, index: usize, visitor: &mut V) -> crate::Result<()> {
        match index.checked_sub(1) {
            Some(new_index) => self.trailing.visit_mut(new_index, visitor),
            None => match &mut self.this {
                Some(this) => this.visit_mut(visitor),
                None => {
                    panic!("attempt to select an uninitialized select-body");
                }
            },
        }
    }
}
