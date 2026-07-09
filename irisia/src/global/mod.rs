use std::{
    any::TypeId,
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

use crate::global::{
    components::CompStock,
    pool::{Pool, PoolId},
};

pub mod components;
pub mod entity;
pub(crate) mod pool;

type CellPool<T> = RefCell<Pool<T>>;

pub struct Runtime {
    // this: Weak<Self>,
    entities: RefCell<Pool<EntityData>>,
    components: RefCell<CompStock>,
}

#[derive(Default)]
pub(crate) struct EntityData {
    components: HashMap<TypeId, PoolId>,
    first_child: Option<EntityObject>,
    next_sibling: Option<EntityObject>,
}

/// 指向一个实体的句柄。
/// 注意：如果该实体不再使用，应该及时调用destroy_entity销毁
#[derive(Clone)]
pub struct EntityObject {
    rt: Weak<Runtime>,
    id: PoolId,
}

impl Runtime {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            // this: weak.clone(),
            entities: Pool::new(128).into(),
            components: CompStock::new().into(),
        })
    }

    pub(crate) fn create_entity(self: &Rc<Self>) -> EntityObject {
        EntityObject {
            rt: Rc::downgrade(self),
            id: self.entities.borrow_mut().insert(EntityData::default()),
        }
    }
}
