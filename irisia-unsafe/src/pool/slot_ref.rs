use std::{
    cell::{Cell, Ref, RefMut},
    ops::{Deref, DerefMut},
    rc::{Rc, Weak},
};

use thiserror::Error;

use super::{destroy_slot_in_place, resolve, Pool, PoolId, PoolSlot, PoolSlotContent};

#[derive(Debug, Error)]
pub enum PoolAccessError {
    #[error("cannot access item because it does not exist")]
    NotFound,
    #[error("cannot access item because it is being borrowed")]
    Borrowed,
}

pub struct SlotRef<T: 'static> {
    // 字段顺序不可颠倒，Ref必须先Drop
    r: Ref<'static, T>,
    guard: CheckRemove<T>,
}

impl<T> Pool<T> {
    pub fn access(&self, id: PoolId) -> Result<SlotRef<T>, PoolAccessError> {
        let (guard, slot) = unsafe { access_base(self, id)? };

        let value_ref = Ref::filter_map(
            slot.content
                .try_borrow()
                .map_err(|_| PoolAccessError::Borrowed)?,
            |content| {
                if let PoolSlotContent::Occupied { value } = content {
                    Some(value)
                } else {
                    None
                }
            },
        )
        .map_err(|_| PoolAccessError::NotFound)?;

        Ok(SlotRef {
            r: value_ref,
            guard,
        })
    }
}

impl<T> Deref for SlotRef<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.r
    }
}

pub struct SlotRefMut<T: 'static> {
    // 字段顺序不可颠倒，Ref必须先Drop
    r: RefMut<'static, T>,
    guard: CheckRemove<T>,
}

impl<T> Pool<T> {
    pub fn access_mut(&self, id: PoolId) -> Result<SlotRefMut<T>, PoolAccessError> {
        let (guard, slot) = unsafe { access_base(self, id)? };

        let value_ref = RefMut::filter_map(
            slot.content
                .try_borrow_mut()
                .map_err(|_| PoolAccessError::Borrowed)?,
            |content| {
                if let PoolSlotContent::Occupied { value } = content {
                    Some(value)
                } else {
                    None
                }
            },
        )
        .map_err(|_| PoolAccessError::NotFound)?;

        Ok(SlotRefMut {
            r: value_ref,
            guard,
        })
    }
}

impl<T> Deref for SlotRefMut<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.r
    }
}

impl<T> DerefMut for SlotRefMut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.r
    }
}

/// SAFETY: PoolSlot仅在Rc有效时有效
unsafe fn access_base<T>(
    this: &Pool<T>,
    id: PoolId,
) -> Result<(CheckRemove<T>, &'static PoolSlot<T>), PoolAccessError> {
    let rc_chunk = this
        .buffer
        .get(id.chunk_index)
        .ok_or(PoolAccessError::NotFound)?;

    let slot = resolve(&this.buffer, id).ok_or(PoolAccessError::NotFound)?;

    let check_remove = CheckRemove {
        chunk: rc_chunk.clone(),
        vacant_chain: Rc::downgrade(&this.first_vacant),
        pid: id,
    };

    Ok((check_remove, unsafe { &*(slot as *const _) }))
}

struct CheckRemove<T> {
    chunk: Rc<[PoolSlot<T>]>,
    vacant_chain: Weak<Cell<Option<PoolId>>>,
    pid: PoolId,
}

impl<T> Drop for CheckRemove<T> {
    fn drop(&mut self) {
        let slot = &self.chunk[self.pid.slot_index];

        if !slot.should_destroy.get() {
            return;
        }

        let Some(vacant_chain) = self.vacant_chain.upgrade() else {
            return;
        };

        destroy_slot_in_place(self.pid, slot, &vacant_chain, true);
    }
}
