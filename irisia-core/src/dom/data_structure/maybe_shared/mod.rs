use std::{
    cell::{Ref as CellRef, RefCell, RefMut as CellRefMut},
    rc::Rc,
};

use self::borrowed::{Ref, RefMut};

pub mod borrowed;

pub enum MaybeShared<T, U> {
    Unique(T),
    Shared(Rc<RefCell<Shared<T, U>>>),
}

pub struct Shared<T, U> {
    pub main: T,
    pub extra: U,
}

impl<T, U> MaybeShared<T, U> {
    pub const fn new(value: T) -> Self {
        Self::Unique(value)
    }

    pub const fn is_unique(&self) -> bool {
        matches!(self, Self::Unique(_))
    }

    pub const fn is_shared(&self) -> bool {
        matches!(self, Self::Shared(_))
    }

    pub fn to_shared(&mut self, extra: U) {
        take_mut::take(self, |this| match this {
            shared @ Self::Shared(_) => shared,
            Self::Unique(val) => Self::Shared(Rc::new(RefCell::new(Shared { main: val, extra }))),
        });
    }

    #[must_use]
    pub fn try_to_unique(&mut self) -> bool {
        let mut result = true;
        take_mut::take(self, |this| match this {
            unique @ Self::Unique(_) => unique,
            Self::Shared(sh) => match Rc::try_unwrap(sh) {
                Ok(ref_cell) => Self::Unique(ref_cell.into_inner().main),
                Err(this) => {
                    result = false;
                    Self::Shared(this)
                }
            },
        });
        result
    }

    pub fn borrow(&self) -> Ref<T> {
        match self {
            Self::Unique(v) => Ref::Unique(v),
            Self::Shared(v) => Ref::Shared(CellRef::map(v.borrow(), |inner| &inner.main)),
        }
    }

    pub fn borrow_mut(&mut self) -> RefMut<T> {
        match self {
            Self::Unique(v) => RefMut::Unique(v),
            Self::Shared(v) => {
                RefMut::Shared(CellRefMut::map(v.borrow_mut(), |inner| &mut inner.main))
            }
        }
    }

    pub fn clone(&mut self, to_shared: impl FnOnce() -> U) -> Self {
        match self {
            Self::Unique(_) => {
                self.to_shared(to_shared());
                match self {
                    Self::Shared(s) => Self::Shared(s.clone()),
                    Self::Unique(_) => unreachable!(),
                }
            }
            Self::Shared(shared) => Self::Shared(shared.clone()),
        }
    }
}
