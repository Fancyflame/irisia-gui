use std::{
    cell::{Ref as CellRef, RefMut as CellRefMut},
    ops::{Deref, DerefMut},
};

pub enum Ref<'a, T> {
    Unique(&'a T),
    Shared(CellRef<'a, T>),
}

impl<T> Deref for Ref<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Unique(v) => &v,
            Self::Shared(v) => &v,
        }
    }
}

pub enum RefMut<'a, T> {
    Unique(&'a mut T),
    Shared(CellRefMut<'a, T>),
}

impl<T> Deref for RefMut<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Unique(v) => &v,
            Self::Shared(v) => &v,
        }
    }
}

impl<T> DerefMut for RefMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Unique(v) => v,
            Self::Shared(v) => &mut *v,
        }
    }
}
