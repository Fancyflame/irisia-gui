use std::{
    any::{Any, TypeId},
    collections::{HashMap, hash_map::Entry},
    ops::{Deref, DerefMut},
};

use anyhow::Result;

use crate::{
    Component,
    global::pool::{Pool, PoolAccessError, PoolId, SlotRef, SlotRefMut},
};

#[derive(Default)]
pub(super) struct CompStock {
    // HashMap<组件类型, Box<Pool<组件>>>
    stock: HashMap<TypeId, Box<dyn UntypedPool>>,
}

impl CompStock {
    pub fn new() -> Self {
        Self::default()
    }

    /// 被get和get_mut使用。获取id对应的组件引用
    fn get_comp_ref_with<T, F, R>(&self, id: &CompId, f: F) -> Result<Option<R>>
    where
        T: Component,
        F: Fn(&Pool<T>) -> Result<R, PoolAccessError>,
    {
        let pool: Option<&Pool<T>> = self
            .stock
            .get(&id.tid)
            .and_then(|boxed| (boxed.as_ref() as &dyn UntypedPool as &dyn Any).downcast_ref());

        let Some(pool) = pool else {
            return Ok(None);
        };

        match f(pool) {
            Ok(r) => Ok(Some(r)),
            Err(PoolAccessError::NotFound) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    /// 获取指定组件引用
    pub fn get<T: Component>(&self, id: &CompId) -> Result<Option<CompRef<T>>> {
        /* let Some(pool) = self.get_pool(id) else {
            return Ok(None);
        };

        match pool.access(id.pid) {
            Ok(r) => Ok(Some(CompRef { inner: r })),
            Err(PoolAccessError::NotFound) => Ok(None),
            Err(err) => Err(err.into()),
        } */
        self.get_comp_ref_with(id, |pool| {
            Ok(CompRef {
                inner: pool.access(id.pid)?,
            })
        })
    }

    /// 获取指定组件可变引用
    pub fn get_mut<T: Component>(&self, id: &CompId) -> Result<Option<CompRefMut<T>>> {
        self.get_comp_ref_with(id, |pool| {
            Ok(CompRefMut {
                inner: pool.access_mut(id.pid)?,
            })
        })
    }

    /// 插入组件
    pub fn insert<T: Component>(&mut self, comp: T) -> CompId {
        let pid = match self.stock.entry(TypeId::of::<T>()) {
            Entry::Vacant(vac) => {
                let mut pool = Pool::new(1);
                let comp_id = pool.insert(comp);
                let rc_pool: Box<Pool<T>> = Box::new(pool);
                vac.insert(rc_pool);
                comp_id
            }
            Entry::Occupied(mut occ) => {
                let pool = (occ.get_mut().as_mut() as &mut dyn Any)
                    .downcast_mut::<Pool<T>>()
                    .unwrap();
                pool.insert(comp)
            }
        };

        CompId {
            pid,
            tid: TypeId::of::<T>(),
        }
    }

    /// 删除组件
    pub fn remove(&mut self, id: &CompId) {
        let Some(pool) = self.stock.get_mut(&id.tid) else {
            return;
        };

        pool.remove_untyped(id.pid);
    }
}

/// 一个组件的ID。需要确保ID和对象池一致
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct CompId {
    pub pid: PoolId,
    pub tid: TypeId,
}

pub struct CompRef<T: 'static> {
    inner: SlotRef<T>,
}

impl<T> Deref for CompRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct CompRefMut<T: 'static> {
    inner: SlotRefMut<T>,
}

impl<T> Deref for CompRefMut<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for CompRefMut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

trait UntypedPool: Any {
    fn remove_untyped(&mut self, id: PoolId);
}

impl<T: 'static> UntypedPool for Pool<T> {
    fn remove_untyped(&mut self, id: PoolId) {
        self.remove(id, true);
    }
}
