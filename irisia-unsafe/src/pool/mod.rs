pub use slot_ref::*;
use std::{
    cell::{Cell, RefCell},
    fmt::Debug,
    rc::Rc,
};

mod slot_ref;

type PoolVec<T> = Vec<Rc<[PoolSlot<T>]>>;

pub struct Pool<T> {
    first_chunk_size: usize,
    first_vacant: Rc<Cell<Option<PoolId>>>,
    buffer: PoolVec<T>,
}

struct PoolSlot<T> {
    // 当被引用时无法及时销毁，需要设置为true
    should_destroy: Cell<bool>,
    version: Cell<u64>,
    content: RefCell<PoolSlotContent<T>>,
}

enum PoolSlotContent<T> {
    Vacant { next_vacant: Option<PoolId> },
    Occupied { value: T },
}

impl<T> PoolSlotContent<T> {
    const EMPTY: Self = Self::Vacant { next_vacant: None };
}

impl<T> Pool<T> {
    pub fn new(first_chunk_size: usize) -> Self {
        assert!(first_chunk_size > 0);
        Pool {
            first_chunk_size,
            first_vacant: Default::default(),
            buffer: Vec::new(),
        }
    }

    pub fn insert(&mut self, value: T) -> PoolId {
        // 如果有空位，则复用空位
        while let Some(pool_id) = self.first_vacant.get() {
            let slot = resolve(&self.buffer, pool_id).unwrap();
            debug_assert!(!slot.should_destroy.get());
            let mut content = slot.content.borrow_mut();

            let PoolSlotContent::Vacant {
                next_vacant: next_empty,
            } = *content
            else {
                unreachable!();
            };
            self.first_vacant.set(next_empty);

            // 如果无法写入下一个版本，则弃用该位置，直接泄漏掉
            let Some(next_version) = slot.version.get().checked_add(1) else {
                continue;
            };

            slot.version.set(next_version);
            *content = PoolSlotContent::Occupied { value };
            return PoolId {
                version: next_version,
                ..pool_id
            };
        }

        // 下一个分配的chunk是现在总容量的2倍
        let next_chunk_capacity =
            self.first_chunk_size * (1 << self.buffer.len().saturating_sub(1));

        let chunk_index = self.buffer.len();
        let chunk = (0..next_chunk_capacity)
            .map(|slot_index| PoolSlot {
                version: Cell::new(0),
                should_destroy: Cell::new(false),
                content: RefCell::new(PoolSlotContent::Vacant {
                    next_vacant: (slot_index + 1 < next_chunk_capacity).then_some(PoolId {
                        chunk_index,
                        slot_index: slot_index + 1,
                        version: 0,
                    }),
                }),
            })
            .collect::<Vec<_>>()
            .into();

        self.buffer.push(chunk);
        self.first_vacant.set(Some(PoolId {
            chunk_index,
            slot_index: 0,
            version: 0,
        }));

        self.insert(value)
    }

    /// 删除对象。如果成功删除则返回Some，如果本身就不存在或无法及时删除则返回None
    pub fn remove(&mut self, id: PoolId, allow_delay: bool) -> Option<T> {
        let slot = resolve(&self.buffer, id)?;
        destroy_slot_in_place(id, slot, &self.first_vacant, allow_delay)
    }
}

#[must_use]
fn resolve<T>(pool: &PoolVec<T>, id: PoolId) -> Option<&PoolSlot<T>> {
    let slot = pool.get(id.chunk_index)?.get(id.slot_index)?;
    (slot.version.get() == id.version).then_some(slot)
}

fn destroy_slot_in_place<T>(
    id: PoolId,
    slot: &PoolSlot<T>,
    vacant_chain: &Cell<Option<PoolId>>,
    allow_delay: bool,
) -> Option<T> {
    let Ok(mut content) = slot.content.try_borrow_mut() else {
        if allow_delay {
            slot.should_destroy.set(true);
            return None;
        } else {
            panic!("cannot delete object immediately as it is borrowed");
        }
    };

    let value = match &*content {
        PoolSlotContent::Occupied { .. } => {
            let PoolSlotContent::Occupied { value } = std::mem::replace(
                &mut *content,
                PoolSlotContent::Vacant {
                    next_vacant: vacant_chain.get(),
                },
            ) else {
                unreachable!();
            };
            value
        }
        PoolSlotContent::Vacant { .. } => {
            return None;
        }
    };

    slot.should_destroy.set(false);
    vacant_chain.set(Some(id));
    Some(value)
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct PoolId {
    chunk_index: usize,
    slot_index: usize,
    version: u64,
}

impl Debug for PoolId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "c{}s{}v{}",
            self.chunk_index, self.slot_index, self.version
        )
    }
}
