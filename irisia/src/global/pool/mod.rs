use anyhow::{Result, anyhow};
pub use slot_ref::*;
use std::{
    cell::{Cell, RefCell},
    fmt::Debug,
    mem,
    rc::Rc,
};

mod slot_ref;

const POOL_DEFAULT_CHUNK_SIZE: usize = 128;
const _: () = const {
    if POOL_DEFAULT_CHUNK_SIZE == 0 {
        panic!("POOL_DEFAULT_CHUNK_SIZE cannot be 0");
    }
};

type PoolVec<T> = Vec<Rc<[PoolSlot<T>]>>;

pub(crate) struct Pool<T> {
    first_vacant: Option<PoolId>,
    buffer: PoolVec<T>,
}

struct PoolSlot<T> {
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
    pub const fn new() -> Self {
        Pool {
            first_vacant: None,
            buffer: Vec::new(),
        }
    }

    pub fn insert(&mut self, value: T) -> PoolId {
        // 如果有空位，则复用空位
        while let Some(pool_id) = self.first_vacant {
            let slot = resolve(&self.buffer, pool_id).unwrap();
            let mut content = slot.content.borrow_mut();

            let PoolSlotContent::Vacant {
                next_vacant: next_empty,
            } = *content
            else {
                unreachable!();
            };
            self.first_vacant = next_empty;

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
            POOL_DEFAULT_CHUNK_SIZE * (1 << self.buffer.len().saturating_sub(1));

        let chunk_index = self.buffer.len();
        let chunk = (0..next_chunk_capacity)
            .map(|slot_index| PoolSlot {
                version: Cell::new(0),
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
        self.first_vacant = Some(PoolId {
            chunk_index,
            slot_index: 0,
            version: 0,
        });

        self.insert(value)
    }

    pub fn remove(&mut self, id: PoolId) -> Result<()> {
        let slot = resolve(&self.buffer, id)?;
        let mut content = slot.content.try_borrow_mut().map_err(|_| {
            anyhow!("cannot remove item because it is being borrowed. Pool ID: {id:?}")
        })?;

        let value = match &*content {
            PoolSlotContent::Occupied { .. } => {
                *content = PoolSlotContent::Vacant {
                    next_vacant: self.first_vacant,
                };
            }
            vacant @ PoolSlotContent::Vacant { .. } => {
                return Ok(());
            }
        };
        self.first_vacant = Some(id);

        Ok(())
    }
}

#[must_use]
const fn resolve<T>(pool: &PoolVec<T>, id: PoolId) -> Option<&PoolSlot<T>> {
    let slot = pool.get(id.chunk_index)?.get(id.slot_index)?;
    (slot.version.get() == id.version).then_some(slot)
}

#[derive(Clone, Copy)]
pub(crate) struct PoolId {
    chunk_index: usize,
    slot_index: usize,
    version: u64,
}

impl Debug for PoolId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;
        write!(
            f,
            "c{}s{}v{}",
            self.chunk_index, self.slot_index, self.version
        )
    }
}
