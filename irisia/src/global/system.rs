use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
};

#[derive(Default)]
pub struct SystemStock {
    map: HashMap<TypeId, Rc<RefCell<dyn Any>>>,
}

impl SystemStock {
    pub fn new() -> Self {
        SystemStock {
            map: Default::default(),
        }
    }
}
