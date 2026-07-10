use std::{
    any::TypeId,
    collections::{HashMap, hash_map::Entry},
    rc::{Rc, Weak},
};

use anyhow::{Result, anyhow, bail};
use smallvec::{SmallVec, smallvec};

use crate::{
    Component,
    global::{
        CompStock, Runtime,
        components::{CompId, CompRef, CompRefMut},
        pool::{PoolId, SlotRefMut},
    },
};

#[derive(Default)]
pub(crate) struct EntityData {
    pub components: HashMap<TypeId, PoolId>,
    pub first_child: Option<EntityObject>,
    pub next_sibling: Option<EntityObject>,
}

/// 指向一个实体的句柄。
/// 注意：如果该实体不再使用，应该及时调用destroy_entity销毁
#[derive(Clone)]
pub struct EntityObject {
    rt: Weak<Runtime>,
    id: PoolId,
}

const REASON_COMPONENT_BORROWED: &str = "this component has been borrowed, please ensure \
    the previous borrow handle has been dropped properly";

impl EntityObject {
    fn entity_mut(&self) -> Option<(Rc<Runtime>, SlotRefMut<EntityData>)> {
        let rt = self.rt.upgrade()?;
        let access = rt.entities.borrow().access_mut(self.id).ok()?;
        Some((rt, access))
    }

    pub fn attach_component<T: Component>(&self, comp: T) -> Result<()> {
        let (rt, mut entity) = self
            .entity_mut()
            .ok_or_else(|| anyhow!("entity does not exist"))?;

        let Entry::Vacant(vac) = entity.components.entry(TypeId::of::<T>()) else {
            bail!("cannot attach duplicate components of the same type");
        };

        let comp_pid = rt
            .components
            .temp_borrow_mut()
            .insert(comp, self.clone())
            .pid;
        vac.insert(comp_pid);
        Ok(())
    }

    pub fn remove_component<T: Component>(&self) {
        let Some((rt, mut entity)) = self.entity_mut() else {
            return;
        };

        let Some((tid, pid)) = entity.components.remove_entry(&TypeId::of::<T>()) else {
            return;
        };

        drop(entity);
        CompStock::remove(rt.components.temp_borrow_mut(), &CompId { pid, tid });
    }

    fn get_comp_id<T: Component>(&self) -> Option<(Rc<Runtime>, CompId)> {
        let rt = self.rt.upgrade()?;

        let entity = rt.entities.borrow().access(self.id).ok()?;

        let tid = TypeId::of::<T>();
        let pid = entity.components.get(&tid)?.clone();

        Some((rt, CompId { pid, tid }))
    }

    pub fn get_component<T: Component>(&self) -> Option<CompRef<T>> {
        let (rt, comp_id) = self.get_comp_id::<T>()?;
        rt.components
            .temp_borrow()
            .get(&comp_id)
            .expect("cannot borrow component as immutable because it has been borrowed as mutable")
    }

    pub fn get_component_mut<T: Component>(&self) -> Option<CompRefMut<T>> {
        let (rt, comp_id) = self.get_comp_id::<T>()?;
        rt.components
            .temp_borrow()
            .get_mut(&comp_id)
            .expect("cannot borrow component as mutable because it has been borrowed")
    }

    pub fn get_global(&self) -> &Weak<Runtime> {
        &self.rt
    }

    pub fn is_valid(&self) -> bool {
        self.entity_mut().is_some()
    }
}

impl Runtime {
    pub(crate) fn create_entity(self: &Rc<Self>) -> EntityObject {
        EntityObject {
            rt: Rc::downgrade(self),
            id: self.entities.borrow_mut().insert(EntityData::default()),
        }
    }

    /// 删除当前实体及其所有子节点，不修复父级和兄弟级关系
    pub(crate) fn destroy_entity(&self, entity_id: PoolId) {
        let Some(root) = self.entities.borrow_mut().remove(entity_id, false) else {
            return;
        };
        let mut queue: SmallVec<[EntityData; 16]> = smallvec![root];

        while let Some(data) = queue.pop() {
            for (tid, pid) in data.components {
                let comp_id = CompId { tid, pid };

                // 注意：这里会执行组件的析构函数
                CompStock::remove(self.components.temp_borrow_mut(), &comp_id);
            }

            let mut entity_to_destroy = data.first_child;
            while let Some(child) = entity_to_destroy {
                let child_data = self
                    .entities
                    .borrow_mut()
                    .remove(child.id, false)
                    .expect("child entity should exist while destroying subtree");
                entity_to_destroy = child_data.next_sibling.as_ref().cloned();
                queue.push(child_data);
            }
        }
    }
}
