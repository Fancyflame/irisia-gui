use std::{any::TypeId, collections::hash_map::Entry, rc::Rc};

use anyhow::{Result, anyhow, bail};
use smallvec::{SmallVec, smallvec};

use crate::{
    Component,
    global::{
        EntityData, EntityObject, Runtime,
        components::{CompId, CompRef, CompRefMut},
        pool::PoolId,
    },
};

impl EntityObject {
    pub fn attach_component<T: Component>(&self, comp: T) -> Result<()> {
        let rt = self
            .rt
            .upgrade()
            .ok_or_else(|| anyhow!("runtime of this entity has destroyed"))?;

        let mut entity = rt.entities.borrow().access_mut(self.id).unwrap();

        let Entry::Vacant(vac) = entity.components.entry(TypeId::of::<T>()) else {
            bail!("cannot attach duplicate components of the same type");
        };

        let comp_pid = rt.components.borrow_mut().insert(comp).pid;
        vac.insert(comp_pid);
        Ok(())
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
            .borrow()
            .get(&comp_id)
            .expect("cannot borrow component as immutable because it has been borrowed as mutable")
    }

    pub fn get_component_mut<T: Component>(&self) -> Option<CompRefMut<T>> {
        let (rt, comp_id) = self.get_comp_id::<T>()?;
        rt.components
            .borrow()
            .get_mut(&comp_id)
            .expect("cannot borrow component as mutable because it has been borrowed")
    }

    pub fn destroy(&self) {
        if let Some(rt) = self.rt.upgrade() {
            rt.destroy_entity(self.id);
        };
    }
}

impl Runtime {
    /// 删除当前实体，返回下一个兄弟节点
    pub(crate) fn destroy_entity(&self, entity_id: PoolId) -> Option<EntityObject> {
        let mut entities = self.entities.borrow_mut();

        // 如果这步返回None，说明这个实体已被删除。
        // 不允许延迟删除，因为这样子树无法清除，且不应该发生引用占用。
        let value = entities.remove(entity_id, false)?;
        let return_value_next_sibling = value.next_sibling.clone();

        // 将相邻兄弟节点和父级关系修复
        {
            if let Some(prev) = &value.prev_sibling {
                entities
                    .access_mut(prev.id)
                    .expect("previous sibling should exist while destroying entity")
                    .next_sibling = value.next_sibling.clone();
            } else if let Some(parent) = &value.parent {
                entities
                    .access_mut(parent.id)
                    .expect("parent should exist while destroying entity")
                    .first_child = value.next_sibling.clone()
            }

            if let Some(next) = &value.next_sibling {
                entities
                    .access_mut(next.id)
                    .expect("next sibling should exist while destroying entity")
                    .prev_sibling = value.prev_sibling.clone();
            }
        }

        let mut queue: SmallVec<[EntityData; 16]> = smallvec![value];

        let mut components = self.components.borrow_mut();
        while let Some(data) = queue.pop() {
            for (tid, pid) in data.components {
                let comp_id = CompId { tid, pid };
                components.remove(&comp_id);
            }

            let mut entity_to_destroy = data.first_child;
            while let Some(child) = entity_to_destroy {
                let child_data = entities
                    .remove(child.id, false)
                    .expect("child entity should exist while destroying subtree");
                entity_to_destroy = child_data.next_sibling.as_ref().cloned();
                queue.push(child_data);
            }
        }

        return_value_next_sibling
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::HashMap, rc::Rc};

    use crate::global::{components::CompStock, pool::Pool};

    use super::*;

    struct TestComponent(u32);

    impl Component for TestComponent {}

    fn entity_ref(rt: &Rc<Runtime>, id: PoolId) -> EntityObject {
        EntityObject {
            rt: Rc::downgrade(rt),
            id,
        }
    }

    fn insert_entity(rt: &Rc<Runtime>) -> EntityObject {
        let id = rt.entities.borrow_mut().insert(EntityData {
            components: HashMap::new(),
            parent: None,
            first_child: None,
            prev_sibling: None,
            next_sibling: None,
        });

        entity_ref(rt, id)
    }

    fn link_children(rt: &Rc<Runtime>, parent: &EntityObject, children: &[&EntityObject]) {
        let mut entities = rt.entities.borrow_mut();
        entities.access_mut(parent.id).unwrap().first_child =
            children.first().map(|child| entity_ref(rt, child.id));

        for (index, child) in children.iter().enumerate() {
            let prev = index
                .checked_sub(1)
                .map(|prev_index| entity_ref(rt, children[prev_index].id));
            let next = children
                .get(index + 1)
                .map(|next_child| entity_ref(rt, next_child.id));

            let mut data = entities.access_mut(child.id).unwrap();
            data.parent = Some(entity_ref(rt, parent.id));
            data.prev_sibling = prev;
            data.next_sibling = next;
        }
    }

    #[test]
    fn destroy_entity_relinks_siblings() {
        let rt = Runtime::new();
        let parent = insert_entity(&rt);
        let child_1 = insert_entity(&rt);
        let child_2 = insert_entity(&rt);
        let child_3 = insert_entity(&rt);

        link_children(&rt, &parent, &[&child_1, &child_2, &child_3]);

        let next = rt.destroy_entity(child_2.id).unwrap();
        assert_eq!(next.id, child_3.id);

        let entities = rt.entities.borrow();
        let parent_data = entities.access(parent.id).unwrap();
        let child_1_data = entities.access(child_1.id).unwrap();
        let child_3_data = entities.access(child_3.id).unwrap();

        assert_eq!(
            parent_data.first_child.as_ref().map(|entity| entity.id),
            Some(child_1.id)
        );
        assert_eq!(
            child_1_data.next_sibling.as_ref().map(|entity| entity.id),
            Some(child_3.id)
        );
        assert_eq!(
            child_3_data.prev_sibling.as_ref().map(|entity| entity.id),
            Some(child_1.id)
        );
        assert!(entities.access(child_2.id).is_err());
    }

    #[test]
    fn destroy_entity_updates_first_child_when_removing_head() {
        let rt = Runtime::new();
        let parent = insert_entity(&rt);
        let child_1 = insert_entity(&rt);
        let child_2 = insert_entity(&rt);

        link_children(&rt, &parent, &[&child_1, &child_2]);

        let next = rt.destroy_entity(child_1.id).unwrap();
        assert_eq!(next.id, child_2.id);

        let entities = rt.entities.borrow();
        let parent_data = entities.access(parent.id).unwrap();
        let child_2_data = entities.access(child_2.id).unwrap();

        assert_eq!(
            parent_data.first_child.as_ref().map(|entity| entity.id),
            Some(child_2.id)
        );
        assert!(child_2_data.prev_sibling.is_none());
    }

    #[test]
    fn destroy_entity_removes_components_with_delay() {
        let rt = Runtime::new();
        let entity = insert_entity(&rt);
        entity.attach_component(TestComponent(7)).unwrap();

        let comp_id = entity.get_comp_id::<TestComponent>().unwrap().1;
        let held_ref = entity.get_component::<TestComponent>().unwrap();

        rt.destroy_entity(entity.id);

        let kept_alive = rt
            .components
            .borrow()
            .get::<TestComponent>(&comp_id)
            .unwrap()
            .unwrap();
        assert_eq!(kept_alive.0, 7);

        drop(kept_alive);
        drop(held_ref);

        assert!(
            rt.components
                .borrow()
                .get::<TestComponent>(&comp_id)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn destroy_entity_removes_entire_deep_subtree_iteratively() {
        let rt = Runtime::new();
        let root = insert_entity(&rt);
        let mut ids = vec![root.id];
        let mut parent = root.id;

        for _ in 0..10_000 {
            let child = insert_entity(&rt);
            {
                let mut entities = rt.entities.borrow_mut();
                entities.access_mut(parent).unwrap().first_child = Some(entity_ref(&rt, child.id));

                let mut child_data = entities.access_mut(child.id).unwrap();
                child_data.parent = Some(entity_ref(&rt, parent));
            }

            parent = child.id;
            ids.push(child.id);
        }

        assert!(rt.destroy_entity(root.id).is_none());

        let entities = rt.entities.borrow();
        for id in ids {
            assert!(entities.access(id).is_err());
        }
    }
}
