use std::{
    any::TypeId,
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

use crate::{
    global::{
        components::CompStock,
        pool::{Pool, PoolId},
        system::SystemStock,
    },
    utils::ReasonCell,
};
use entity::{EntityData, EntityObject};

pub mod components;
pub mod entity;
pub(crate) mod pool;
pub mod system;

pub struct Runtime {
    // this: Weak<Self>,
    entities: RefCell<Pool<EntityData>>,
    components: ReasonCell<CompStock>,
    system: SystemStock,
}

impl Runtime {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            // this: weak.clone(),
            entities: Pool::new(64).into(),
            components: CompStock::new().into(),
            system: Default::default(),
        })
    }
}
