use std::{
    cell::{Ref, RefMut},
    ops::{Deref, DerefMut},
    rc::Rc,
};

use thiserror::Error;

use crate::global::pool::{Pool, PoolId, PoolSlot, PoolSlotContent, resolve};

#[derive(Debug, Error)]
pub(crate) enum PoolAccessError {
    #[error("cannot access item because it does not exist")]
    NotFound,
    #[error("cannot access item because it is being borrowed")]
    Borrowed,
}

pub struct SlotRef<T> {
    // 字段顺序不可颠倒，Ref必须先Drop
    r: Ref<'static, T>,
    slot: Rc<[PoolSlot<T>]>,
}

impl<T> Pool<T> {
    pub fn access(&self, id: PoolId) -> Result<SlotRef<T>, PoolAccessError> {
        let (rc_chunk, slot) = unsafe { access_base(self, id)? };

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
            slot: rc_chunk,
        })
    }
}

impl<T> Deref for SlotRef<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.r
    }
}

pub struct SlotRefMut<T> {
    // 字段顺序不可颠倒，Ref必须先Drop
    r: RefMut<'static, T>,
    slot: Rc<[PoolSlot<T>]>,
}

impl<T> Pool<T> {
    pub fn access_mut(&self, id: PoolId) -> Result<SlotRefMut<T>, PoolAccessError> {
        let (rc_chunk, slot) = unsafe { access_base(self, id)? };

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
            slot: rc_chunk,
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
) -> Result<(Rc<[PoolSlot<T>]>, &'static PoolSlot<T>), PoolAccessError> {
    (|| {
        let rc_chunk = this.buffer.get(id.chunk_index)?;

        let slot = resolve(&this.buffer, id)?;

        Some((rc_chunk.clone(), unsafe { &*(slot as *const _) }))
    })()
    .ok_or(PoolAccessError::NotFound)
}
