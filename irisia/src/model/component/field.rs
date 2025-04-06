use std::{
    cell::{Cell, Ref, RefCell},
    ops::Deref,
    rc::Rc,
};

pub enum Field<T> {
    SharedRef(Rc<RefCell<T>>),
    Ownership(T),
}

enum ReadFieldInner<'a, T> {
    SharedRef(Ref<'a, T>),
    Ownership(&'a T),
}
